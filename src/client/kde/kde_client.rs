use crate::client::kde::kwin_scripts::KwinScripts;
use crate::client::kde::plugin_script_handler::ensure_script_loaded;
use crate::client::{Client, WindowInfo};
use anyhow::{bail, Result};
use futures::executor::block_on;
use log::{debug, error, warn};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zbus::connection::Builder;
use zbus::{interface, Connection};

pub const KWIN_SCRIPT: &str = include_str!("kwin-script.js");
pub const KWIN_SCRIPT_PLUGIN_NAME: &str = "xremap";

pub struct KdeClient {
    active_window: Arc<Mutex<ActiveWindow>>,
    log_window_changes: bool,
}

impl KdeClient {
    pub fn new(log_window_changes: bool) -> KdeClient {
        let active_window = Arc::new(Mutex::new(ActiveWindow::default()));
        KdeClient {
            active_window,
            log_window_changes,
        }
    }

    fn connect(&mut self) -> Result<()> {
        let active_window = Arc::clone(&self.active_window);
        let log_window_changes = self.log_window_changes;
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let connect = move || -> Result<Connection> {
                let awi = DbusServerInterface {
                    active_window,
                    log_window_changes,
                };

                let connection = Builder::session()?
                    .name("com.k0kubun.Xremap")?
                    .serve_at("/com/k0kubun/Xremap", awi)?
                    .build();

                Ok(block_on(connection)?)
            };

            match connect() {
                Ok(_) => {
                    tx.send(Ok(())).unwrap();
                    loop {
                        thread::sleep(Duration::from_secs(86400));
                    }
                }
                Err(err) => tx.send(Err(err)).unwrap(),
            }
        });

        // Wait for server to start
        rx.recv().unwrap()?;

        // Is only loaded if not already running.
        ensure_script_loaded()?;

        let oneoff_scripts = KwinScripts::new();

        // The script sends a message right away, so it's started after the server.
        if let Err(err) = oneoff_scripts.send_active_window_script_once() {
            // To avoid the risk of breaking change, the error is just printed.
            error!("{err:?}")
        }

        // Busy wait 100ms, so the first use returns a valid value.
        // Testing shows it takes around 10ms to get a response.
        // Notes on correctness:
        //  The lock is just created, so this thread cannot hold the lock already.
        //  `try_lock` doesn't block if the lock is wrongfully held indefinitely by
        //  the other thread, so we are guaranteed to timeout as expected.
        for i in 0..10 {
            if let Ok(aw) = self.active_window.try_lock() {
                if !aw.title.is_empty() {
                    debug!("Connected to KDE within: {}ms", i * 10);
                    return Ok(());
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        debug!("Connection to KDE was not established within 100ms");

        Ok(())
    }
}

impl Client for KdeClient {
    fn supported(&mut self) -> bool {
        let conn_res = self.connect();
        if let Err(err) = &conn_res {
            warn!("Could not connect to kwin-script. Error: {err:?}");
        }
        conn_res.is_ok()
    }

    fn current_window(&mut self) -> Option<String> {
        let aw = self.active_window.lock().ok()?;
        Some(aw.title.clone())
    }

    fn current_application(&mut self) -> Option<String> {
        let aw = self.active_window.lock().ok()?;
        Some(aw.res_class.clone())
    }

    fn window_list(&mut self) -> Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for KDE")
    }

    fn close_windows_by_app_class(&mut self, _app_class: &str) -> Result<()> {
        todo!()
    }
}

#[derive(Default)]
pub struct ActiveWindow {
    res_class: String,
    title: String,
}

struct DbusServerInterface {
    active_window: Arc<Mutex<ActiveWindow>>,
    log_window_changes: bool,
}

#[interface(name = "com.k0kubun.Xremap")]
impl DbusServerInterface {
    fn notify_active_window(&mut self, title: String, res_class: String) {
        // Print when log_window_changes is enabled to help identify application resource classes.
        if self.log_window_changes {
            println!("active window: caption: '{title}', class: '{res_class}'");
        }
        let mut aw = self.active_window.lock().unwrap();
        aw.title = title;
        aw.res_class = res_class;
    }
}
