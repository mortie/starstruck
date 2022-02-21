#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(not(unix))]
mod unix;
#[cfg(not(unix))]
pub use unix::*;

pub use whoami::username;
