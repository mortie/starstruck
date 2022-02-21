use std::ffi::CStr;
use std::os::raw::c_char;
use terminal_size;
use whoami;

extern "C" {
    fn getlogin() -> *const c_char;
}

pub fn term_size() -> (i32, i32) {
    use terminal_size::{Height, Width};
    // We use stderr (FD 2) because the shell messes with stdin and stdout
    match terminal_size::terminal_size_using_fd(2) {
        Some((Width(w), Height(h))) => (w as i32, h as i32),
        None => (80, 60),
    }
}

pub fn login_name() -> String {
    let cstr = unsafe { CStr::from_ptr(getlogin()) };
    match cstr.to_str() {
        Ok(s) => s.to_owned(),
        Err(_) => whoami::username(),
    }
}

#[cfg(not(target_os = "macos"))]
pub use whoami::hostname;

#[cfg(target_os = "macos")]
pub fn hostname() -> String {
    let un = whoami::hostname();
    match un.strip_suffix(".local") {
        Some(un) => un.to_string(),
        _ => un,
    }
}
