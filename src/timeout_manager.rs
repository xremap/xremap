use nix::sys::time::TimeSpec;
#[cfg(target_os = "linux")]
use nix::sys::timerfd::{ClockId, Expiration, TimerFd, TimerFlags, TimerSetTimeFlags};
use std::cell::RefCell;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::{Duration, Instant};

static RESOLUTION: Duration = Duration::from_millis(1);

#[derive(Debug)]
struct State {
    #[cfg(target_os = "linux")]
    timer_fd: RawFd,
    #[cfg(target_os = "linux")]
    timer: TimerFd,
    delays: Vec<Instant>,
}

#[derive(Debug)]
pub struct TimeoutManager {
    state: RefCell<State>,
}

impl TimeoutManager {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
        Self {
            state: RefCell::new(State {
                #[cfg(target_os = "linux")]
                timer_fd: timer.as_raw_fd(),
                #[cfg(target_os = "linux")]
                timer,
                delays: vec![],
            }),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn get_timer_fd(&self) -> RawFd {
        self.state.borrow().timer_fd
    }

    pub fn set_timeout(&self, delay: Duration) -> nix::Result<()> {
        #[cfg(target_os = "linux")]
        return set_timeout(&mut self.state.borrow_mut(), delay);
        #[cfg(target_os = "freebsd")]
        panic!("Double tap and chords are not supported on FreeBSD");
    }

    pub fn need_timeout(&self) -> anyhow::Result<bool> {
        #[cfg(target_os = "freebsd")]
        return Ok(false);
        #[cfg(target_os = "linux")]
        need_timeout(&mut self.state.borrow_mut())
    }
}

#[cfg(target_os = "linux")]
fn set_timeout(state: &mut State, delay: Duration) -> nix::Result<()> {
    set_timer(state)?;

    state.delays.push(Instant::now() + delay);

    Ok(())
}

#[cfg(target_os = "linux")]
fn need_timeout(state: &mut State) -> anyhow::Result<bool> {
    let now = Instant::now();

    let need_timeout = state.delays.iter().any(|timeout_inst| timeout_inst <= &now);

    state.delays.retain(|&timeout_inst| timeout_inst > now);

    if state.delays.is_empty() {
        state.timer.unset()?;
    } else {
        set_timer(state)?;
    }

    Ok(need_timeout)
}

#[cfg(target_os = "linux")]
// Could pick the minimal delay to avoid unneeded ticks.
fn set_timer(state: &mut State) -> nix::Result<()> {
    state
        .timer
        .set(Expiration::OneShot(TimeSpec::from_duration(RESOLUTION)), TimerSetTimeFlags::empty())
}
