use libc::{c_char, c_int, pid_t};
use std::{ffi::CString, io, os::fd::{AsRawFd, RawFd}};

unsafe extern "C" {
    static environ: *const *const c_char;
}

pub enum ForkReturn {
    Parent(pid_t),
    Child,
}

pub(crate) fn fork() -> io::Result<ForkReturn> {
    let res = unsafe { libc::fork() };

    // TODO: Use `assert!` (or `assert_eq!`) here to make sure we only have one thread

    match res {
        ..0 => Err(io::Error::last_os_error()),
        0 => Ok(ForkReturn::Child),
        _ => Ok(ForkReturn::Parent(res)),
    }
}

pub(crate) fn exec<S: AsRef<str>>(pathname: &S, argv: &[&S]) -> io::Result<()> {
    let pathname = CString::new(pathname.as_ref()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "BAD: pathname str had a null byte."))?;

    // Store our CStrings
    let argv = argv
        .iter()
        .map(|arg| CString::new(arg.as_ref()))
        .filter_map(|res| res.ok())
        .collect::<Vec<_>>();

    let mut argv_ptrs = argv
        .iter()
        .map(|arg| arg.as_ptr())
        .collect::<Vec<_>>();
    argv_ptrs.push(std::ptr::null());

    if unsafe { libc::execvpe(pathname.as_ptr(), argv_ptrs.as_ptr(), environ) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        unsafe {
            std::hint::unreachable_unchecked();
        }
    }
}

pub(crate) struct WaitReturn {
    pid: pid_t,
    status: WaitStatus,
}

pub(crate) enum WaitStatus {
    Exited(i32),
    TermSignal(i32),
    Stopped(i32),
    Continued,
    Unknown
}

pub(crate) fn wait() -> io::Result<WaitReturn> {
    use WaitStatus as WS;
    use libc::{WIFEXITED, WEXITSTATUS, WIFSIGNALED, WTERMSIG, WIFSTOPPED, WSTOPSIG, WIFCONTINUED};

    let mut stat_code = 0i32;

    let res = unsafe { libc::wait(&raw mut stat_code) };

    if res < 0 {
        Err(io::Error::last_os_error())
    } else {
        let pid = res;

        let status = if WIFEXITED(stat_code) {
            WS::Exited(WEXITSTATUS(stat_code))
        } else if WIFSIGNALED(stat_code) {
            WS::TermSignal(WTERMSIG(stat_code))
        } else if WIFSTOPPED(stat_code) {
            WS::Stopped(WSTOPSIG(stat_code))
        } else if WIFCONTINUED(stat_code) {
            WS::Continued
        } else {
            WS::Unknown
        };

        Ok(WaitReturn{pid, status})
    }
}

pub struct Pipe {
    pub read_fd: RawFd,
    pub write_fd: RawFd,
}
pub(crate) fn pipe() -> io::Result<Pipe> {
    let mut pipe_fds: [c_int; 2] = [0; 2];

    if unsafe { libc::pipe(pipe_fds.as_mut_ptr()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(Pipe{
            read_fd: pipe_fds[0],
            write_fd: pipe_fds[1],
        })
    }
}

pub(crate) fn dup2<F: AsRawFd>(oldfd: F, newfd: F) -> io::Result<()> {
    if unsafe { libc::dup2(oldfd.as_raw_fd(), newfd.as_raw_fd()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn close<F: AsRawFd>(fd: F) -> io::Result<()> {
    if unsafe { libc::close(fd.as_raw_fd()) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
