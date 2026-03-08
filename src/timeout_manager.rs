use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{ClockId, Expiration, TimerFd, TimerFlags, TimerSetTimeFlags};
use std::cell::RefCell;
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::{Duration, Instant};

static RESOLUTION: Duration = Duration::from_millis(1);

#[derive(Debug)]
struct State {
    timer_fd: RawFd,
    timer: TimerFd,
    delays: Vec<Instant>,
}

#[derive(Debug)]
pub struct TimeoutManager {
    state: RefCell<State>,
}

impl TimeoutManager {
    pub fn new() -> Self {
        let timer = TimerFd::new(ClockId::CLOCK_MONOTONIC, TimerFlags::empty()).unwrap();
        Self {
            state: RefCell::new(State {
                timer_fd: timer.as_raw_fd(),
                timer,
                delays: vec![],
            }),
        }
    }

    pub fn get_timer_fd(&self) -> RawFd {
        self.state.borrow().timer_fd
    }

    pub fn set_timeout(&self, delay: Duration) -> nix::Result<()> {
        set_timeout(&mut self.state.borrow_mut(), delay)
    }

    pub fn need_timeout(&self) -> anyhow::Result<bool> {
        need_timeout(&mut self.state.borrow_mut())
    }
}

fn set_timeout(state: &mut State, delay: Duration) -> nix::Result<()> {
    set_timer(state)?;

    state.delays.push(Instant::now() + delay);

    Ok(())
}

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

// Could pick the minimal delay to avoid unneeded ticks.
fn set_timer(state: &mut State) -> nix::Result<()> {
    state
        .timer
        .set(Expiration::OneShot(TimeSpec::from_duration(RESOLUTION)), TimerSetTimeFlags::empty())
}
