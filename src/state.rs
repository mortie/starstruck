#[derive(Copy, Clone)]
pub enum Shell {None, Bash, Zsh}

pub struct State {
    pub exit_code: u8,
    pub shell: Shell,
}

impl Shell {
    pub fn escape_start(self) -> &'static str {
        match self {
            Shell::None => "",
            Shell::Bash => "\\[",
            Shell::Zsh => "%{",
        }
    }

    pub fn escape_end(self) -> &'static str {
        match self {
            Shell::None => "",
            Shell::Bash => "\\]",
            Shell::Zsh => "%}",
        }
    }
}
