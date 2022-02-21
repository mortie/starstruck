use whoami;

pub fn term_size() -> (i32, i32) {
    (80, 60)
}

pub fn login_name() -> String {
    whomai::username()
}

pub use whoami::hostname;
