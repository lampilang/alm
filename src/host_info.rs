#[cfg(target_os = "windows")]
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};

#[cfg(target_os = "linux")]
use libc::{sysconf, _SC_NPROCESSORS_ONLN};

#[cfg(
    any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )
)]
use {
    libc::{c_void, sysctlbyname},
    std::mem::{size_of, uninitialized},
    std::ptr::null,
};

#[cfg(target_os = "linux")]
pub fn cpu_count() -> Option<usize> {
    Some(unsafe { sysconf(_SC_NPROCESSORS_ONLN) } as usize)
}

#[cfg(target_os = "windows")]
pub fn cpu_count() -> Option<usize> {
    Some(unsafe {
        let mut info = uninitialized::<SYSTEM_INFO>();
        GetSystemInfo(&mut info as *mut _);
        info.dwNumberOfProcessors
    } as usize)
}

#[cfg(
    any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        otarget_os = "openbsd"
    )
)]
pub fn cpu_count() -> Option<usize> {
    Some(unsafe {
        let mut num = uninitialized();
        let mut size = size_of::<usize>();
        sysctlbyname(
            b"hw.ncpu" as *const u8 as *const i8,
            &mut num as *mut _ as *mut c_void,
            &mut size as *mut _,
            null(),
            0,
        );
        num
    })
}

#[cfg(
    not(
        any(
            target_os = "linux",
            target_os = "windows",
            target_os = "macos",
            target_os = "freebsd",
            target_os = "netbsd",
            otarget_os = "openbsd"
        )
    )
)]
pub fn cpu_count() -> Option<usize> { None }
