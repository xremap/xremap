use nix::sys::time::TimeSpec;
#[cfg(target_os = "linux")]
use nix::sys::timerfd::{ClockId, Expiration, TimerFd, TimerFlags, TimerSetTimeFlags};
use std::cell::RefCell;
#[cfg(target_os = "linux")]
use std::os::fd::{AsFd, BorrowedFd};
use std::time::{Duration, Instant};

static RESOLUTION: Duration = Duration::from_millis(1);

#[derive(Debug)]
pub struct TimeoutManager {
    #[cfg(target_os = "linux")]
    timer: TimerFd,
    delays: RefCell<Vec<Instant>>,
}

impl TimeoutManager {
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "linux")]
            timer: TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap(),
            delays: RefCell::new(vec![]),
        }
    }

    pub fn set_timeout(&self, delay: Duration) -> nix::Result<()> {
        #[cfg(target_os = "linux")]
        return set_timeout(&mut self.delays.borrow_mut(), delay, &self.timer);
        #[cfg(target_os = "freebsd")]
        panic!("Double tap and chords are not supported on FreeBSD");
    }

    pub fn need_timeout(&self) -> anyhow::Result<bool> {
        #[cfg(target_os = "freebsd")]
        return Ok(false);
        #[cfg(target_os = "linux")]
        need_timeout(&mut self.delays.borrow_mut(), &self.timer)
    }
}

#[cfg(target_os = "linux")]
impl AsFd for TimeoutManager {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.timer.as_fd()
    }
}

#[cfg(target_os = "linux")]
fn set_timeout(delays: &mut Vec<Instant>, delay: Duration, timer: &TimerFd) -> nix::Result<()> {
    set_timer(timer)?;

    delays.push(Instant::now() + delay);

    Ok(())
}

#[cfg(target_os = "linux")]
fn need_timeout(delays: &mut Vec<Instant>, timer: &TimerFd) -> anyhow::Result<bool> {
    let now = Instant::now();

    let need_timeout = delays.iter().any(|timeout_inst| timeout_inst <= &now);

    delays.retain(|&timeout_inst| timeout_inst > now);

    if delays.is_empty() {
        timer.unset()?;
    } else {
        set_timer(timer)?;
    }

    Ok(need_timeout)
}

#[cfg(target_os = "linux")]
// Could pick the minimal delay to avoid unneeded ticks.
fn set_timer(timer: &TimerFd) -> nix::Result<()> {
    timer.set(Expiration::OneShot(TimeSpec::from_duration(RESOLUTION)), TimerSetTimeFlags::empty())
}
