extern crate evdev;

use evdev::Device;
use std::os::unix::io::{AsRawFd, RawFd};
use std::{io, mem, ptr};
use std::error::Error;

// A call of this function blocks until the argument device becomes readable.
// TODO: support selecting multiple devices
pub fn is_readable(device: &mut Device) -> Result<bool, Box<dyn Error>> {
    let mut fd_set = FdSet::new(); // TODO: maybe this should be prepared in the caller
    fd_set.set(device.as_raw_fd());

    let result = match unsafe {
        libc::select(
            device.as_raw_fd() + 1,
            to_fdset_ptr(Some(&mut fd_set)),
            to_fdset_ptr(None),
            to_fdset_ptr(None),
            to_ptr::<libc::timeval>(None) as *mut libc::timeval,
        )
    } {
        -1 => Err(io::Error::last_os_error()),
        res => Ok(res as usize),
    }?;
    return Ok(result == 1 && fd_set.is_set(device.as_raw_fd()));
}

fn to_fdset_ptr(opt: Option<&mut FdSet>) -> *mut libc::fd_set {
    match opt {
        None => ptr::null_mut(),
        Some(&mut FdSet(ref mut raw_fd_set)) => raw_fd_set,
    }
}

fn to_ptr<T>(opt: Option<&T>) -> *const T {
    match opt {
        None => ptr::null::<T>(),
        Some(p) => p,
    }
}

struct FdSet(libc::fd_set);

impl FdSet {
    fn new() -> FdSet {
        unsafe {
            // Is the memory released properly?
            let mut raw_fd_set = mem::MaybeUninit::<libc::fd_set>::uninit();
            libc::FD_ZERO(raw_fd_set.as_mut_ptr());
            FdSet(raw_fd_set.assume_init())
        }
    }

    fn set(&mut self, fd: RawFd) {
        unsafe { libc::FD_SET(fd, &mut self.0) }
    }

    fn is_set(&mut self, fd: RawFd) -> bool {
        unsafe { libc::FD_ISSET(fd, &mut self.0) }
    }
}
