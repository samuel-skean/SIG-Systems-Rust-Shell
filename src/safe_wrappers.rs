use libc::{pid_t, c_char};
use std::{ffi::{CStr, CString}, io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult}};

unsafe extern "C" {
    static environ: *const *const c_char;
}

enum ForkReturn {
    Parent(pid_t),
    Child,
}

pub(crate) fn fork() -> IOResult<ForkReturn> {
    let res = unsafe { libc::fork() };

    // TODO: Use `assert!` (or `assert_eq!`) here to make sure we only have one thread

    if res < 0 {
        Err(IOError::last_os_error())
    } else {
         if res == 0 {
            Ok(ForkReturn::Child)
         } else {
            Ok(ForkReturn::Parent(res))
         }
    }
}

pub(crate) fn execve<S: AsRef<str>>(pathname: &S, argv: &[S]) -> IOResult<()> {
    let pathname = CString::new(pathname.as_ref()).map_err(|_| IOError::new(IOErrorKind::InvalidInput, "BAD: pathname str had a null byte."))?;

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

    if unsafe { libc::execve(pathname.as_ptr(), argv_ptrs.as_ptr(), environ) } < 0 {
        Err(IOError::last_os_error())
    } else {
        unsafe {
            std::hint::unreachable_unchecked();
        }
    }
}
