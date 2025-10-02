use anyhow::bail;
use evdev::{Device, FetchEventsSynced, InputEvent};
use nix::sys::select::{select, FdSet};
use nix::sys::time::TimeValLike;
use std::cell::Cell;
use std::iter::repeat_with;
use std::os::unix::io::AsRawFd;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use xremap::device::SEPARATOR;
use xremap::util::until;

use crate::common::{get_random_device_name, get_virtual_device, wait_for_device, wait_for_grabbed, VirtualDeviceInfo};

pub struct XremapBuilder {
    nocapture_: bool,
    log_level_: String,
    // None means open a new input device
    custom_input_device_: Option<String>,
    open_for_fetch_: bool,
    watch_: bool,
}

impl XremapBuilder {
    fn new() -> Self {
        Self {
            nocapture_: false,
            log_level_: "info".into(),
            custom_input_device_: None,
            // If output from xremap isn't grabbed, the events
            // goes to the 'host', so disable with care.
            open_for_fetch_: true,
            watch_: false,
        }
    }

    pub fn nocapture(&mut self) -> &mut Self {
        self.nocapture_ = true;
        self
    }

    pub fn log_level(&mut self, log_level: impl Into<String>) -> &mut Self {
        self.log_level_ = log_level.into();
        self
    }

    pub fn custom_input_device(&mut self, name: impl Into<String>) -> &mut Self {
        self.custom_input_device_ = Some(name.into());
        self
    }

    pub fn not_open_for_fetch(&mut self) -> &mut Self {
        self.open_for_fetch_ = false;
        self
    }
    pub fn watch(&mut self, value: bool) -> &mut Self {
        self.watch_ = value;
        self
    }

    pub fn build(&mut self) -> anyhow::Result<XremapController> {
        XremapController::from_builder(self)
    }
}

#[derive(Debug)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
}

// Devices are managed tightly to avoid possible
// destructive events emitted to the 'host'.
pub struct XremapController {
    // Is None when xremap has been stopped.
    child: Cell<Option<Child>>,
    nocapture: bool,
    // Input from xremap's perspective
    input_device: Option<VirtualDeviceInfo>,
    // Output from xremap's perspective
    output_device_name: String,
    output_device: Option<Device>,
    device_filter: String,
}

impl XremapController {
    pub fn builder() -> XremapBuilder {
        XremapBuilder::new()
    }

    pub fn new() -> anyhow::Result<Self> {
        XremapController::builder().build()
    }

    fn from_builder(def: &XremapBuilder) -> anyhow::Result<Self> {
        let mut command = Command::new("target/debug/xremap");

        let output_device_name =
            format!("test output device {}", repeat_with(fastrand::alphanumeric).take(10).collect::<String>());

        let builder = command
            .env("RUST_LOG", &def.log_level_)
            .args(vec!["--output-device-name", &output_device_name])
            .arg("tests/common/config-test.yml");

        if !def.nocapture_ {
            // Can remove these to get stdio from xremap
            // inline with the stdio from test cases.
            // That makes it easier to debug.
            // But some tests assert on the stdio so they will fail
            // when stdio isn't buffered.
            builder.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        let mut input_device: Option<VirtualDeviceInfo> = None;

        if def.watch_ {
            builder.arg("--watch");
        }

        let device_filter = match def.custom_input_device_.clone() {
            Some(name) => name,
            None => {
                // Use a unique device for xremap to grab
                // so test cases can run in parallel.
                let name = get_random_device_name();

                input_device = Some(get_virtual_device(&name)?);

                name
            }
        };

        // There must always be a device filter, otherwise
        // would test cases try to grab physical devices.
        builder.arg("--device").arg(&device_filter);

        let child = builder.spawn()?;

        match &input_device {
            None => {
                println!("No input device configured for xremap.");
            }
            Some(input_device) => {
                wait_for_grabbed(&input_device.path)?;

                println!("Input device grabbed by xremap");
            }
        }

        let mut ctrl = Self {
            child: Cell::new(Some(child)),
            nocapture: def.nocapture_,
            input_device,
            output_device_name,
            output_device: None,
            device_filter,
        };

        // Default is to grab the device xremap opens to avoid
        // possibly destructive events to be send to the 'host'.
        if def.open_for_fetch_ {
            ctrl.open_output_device()?;
        }

        Ok(ctrl)
    }

    pub fn get_input_device_name<'a>(&'a mut self) -> &'a str {
        &self.device_filter
    }

    pub fn open_input_device(&mut self, name: impl Into<String>) -> anyhow::Result<()> {
        if self.input_device.is_some() {
            bail!("Input device already opened.")
        }

        let dev_info = get_virtual_device(name)?;

        wait_for_grabbed(&dev_info.path)?;

        println!("Input device grabbed by xremap");

        self.input_device = Some(dev_info);

        Ok(())
    }

    pub fn close_input_device(&mut self) -> anyhow::Result<()> {
        if self.input_device.is_none() {
            bail!("Input device not opened.")
        }

        self.input_device = None;
        println!("Input device closed");

        Ok(())
    }

    pub fn open_output_device(&mut self) -> anyhow::Result<()> {
        if self.output_device.is_some() {
            bail!("Output device already opened.")
        }

        let (_, mut device) = wait_for_device(&self.output_device_name)?;

        device.grab()?;

        println!("Output device from xremap grabbed");

        self.output_device = Some(device);

        Ok(())
    }

    pub fn emit_events(&mut self, events: &[InputEvent]) -> anyhow::Result<()> {
        let input_device = self.input_device.as_mut().expect("Input device is not opened");

        let mut probe_device = Device::open(&input_device.path)?;

        if probe_device.grab().is_ok() {
            // Emitting events here would go to the 'host'
            // because xremap has not grabbed the device.
            probe_device.ungrab()?;
            bail!("Input device not grabbed.");
        };

        Ok(input_device.device.emit(events)?)
    }

    pub fn fetch_events(&mut self) -> anyhow::Result<FetchEventsSynced<'_>> {
        let device = self.output_device.as_mut().expect("Output device is not opened");

        let mut fds = FdSet::new();
        let fd = device.as_raw_fd();
        fds.insert(fd);

        select(None, &mut fds, None, None, Some(&mut TimeValLike::seconds(1)))?;

        if !fds.contains(fd) {
            bail!("Timed out waiting for xremap events.");
        }

        Ok(device.fetch_events()?)
    }

    pub fn kill_for_output(&mut self) -> anyhow::Result<Output> {
        self.raw_kill()?;
        self.wait_for_output()
    }

    pub fn wait_for_output(&self) -> anyhow::Result<Output> {
        if self.nocapture {
            bail!("Can't get output when configured for nocapture.");
        }

        let child = self.child.take().expect("Output is already fetched.");

        self.wait_for_output_inner(child)
    }

    /// Expects xremap to stop by itself.
    fn wait_for_output_inner(&self, mut child: Child) -> anyhow::Result<Output> {
        println!("Waiting for xremap to exit");

        let is_stopped = until(
            || child.try_wait().is_ok_and(|val| !val.is_none()),
            Duration::from_secs(1),
            "Timed out waiting for xremap exit",
        );

        if is_stopped.is_err() {
            child.kill()?;
            println!("Xremap killed");
        };

        let res = child.wait_with_output()?;

        println!("Xremap stopped");

        let stdout = String::from_utf8(res.stdout)?;
        let stderr = String::from_utf8(res.stderr)?;

        if self.nocapture {
            // No output to print because of nocapture.
            assert_eq!("", stdout);
            assert_eq!("", stderr);
        } else {
            // To make debugging easier always print output.
            println!("{SEPARATOR}");
            println!("stdout: {stdout}");
            println!("{SEPARATOR}");
            println!("stderr: {stderr}");
            println!("{SEPARATOR}");
        }

        match is_stopped {
            Ok(_) => Ok(Output { stdout, stderr }),
            Err(e) => Err(e),
        }
    }

    pub fn raw_kill(&self) -> anyhow::Result<()> {
        let mut child = self.child.take().expect("Output is already fetched.");

        let result = child.kill();

        println!("Xremap killed");

        self.child.set(Some(child));

        Ok(result?)
    }

    // Kill and ignore stdio
    pub fn kill(&self) -> anyhow::Result<()> {
        if let Some(mut child) = self.child.take() {
            if child.try_wait()?.is_none() {
                child.kill()?;

                println!("Xremap killed");
            }
        }

        Ok(())
    }
}

// Ensures stdio is printed in case test cases fail unexpectedly.
// It also ensures xremap is stopped before the output_device
// is ungrabbed so events don't go to the 'host'.
impl Drop for XremapController {
    fn drop(&mut self) {
        println!("XremapController dropped");
        self.child.take().map(|child| {
            match self.wait_for_output_inner(child) {
                Ok(_) => {
                    // Has already been printed.
                }
                Err(err) => {
                    println!("While dropping xremap command: {err:?}");
                }
            }
        });
    }
}
