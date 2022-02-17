use glisp::eval::{Scope, ValRef};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

struct GitCtx {
    has_searched_gitdir: bool,
    gitdir: Option<PathBuf>,
}

impl GitCtx {
    fn new() -> Self {
        Self {
            has_searched_gitdir: false,
            gitdir: None,
        }
    }

    fn find_gitdir(&mut self) -> bool {
        if self.has_searched_gitdir {
            return self.gitdir.is_some();
        }

        let cwd = match env::current_dir() {
            Ok(dir) => dir,
            Err(..) => {
                self.gitdir = None;
                return false;
            }
        };

        let mut vec = Vec::from_iter(cwd.components());
        while vec.len() > 0 {
            let mut path = PathBuf::new();
            for c in &vec {
                path.push(c)
            }

            path.push(".git");
            match fs::metadata(&path) {
                Ok(meta) => {
                    if meta.is_dir() {
                        self.has_searched_gitdir = true;

                        // Pretend we don't have a .git dir if it's invalid UTF-8,
                        // for simplicity
                        match path.to_str() {
                            None => return false,
                            Some(..) => {
                                self.gitdir = Some(path);
                                return true;
                            }
                        };
                    }
                }
                Err(..) => (),
            };

            vec.pop();
        }

        self.has_searched_gitdir = true;
        false
    }
}

fn has_git(ctx: &Rc<RefCell<GitCtx>>) -> Result<ValRef, String> {
    if ctx.borrow_mut().find_gitdir() {
        Ok(ValRef::Number(1))
    } else {
        Ok(ValRef::Number(0))
    }
}

fn git_dir(ctx: &Rc<RefCell<GitCtx>>) -> Result<ValRef, String> {
    ctx.borrow_mut().find_gitdir();
    match &ctx.borrow().gitdir {
        Some(s) => Ok(ValRef::String(Rc::new(s.to_string_lossy().to_string()))),
        None => Ok(ValRef::None),
    }
}

fn git_branch(ctx: &Rc<RefCell<GitCtx>>) -> Result<ValRef, String> {
    ctx.borrow_mut().find_gitdir();
    let mut path = match &ctx.borrow().gitdir {
        None => return Ok(ValRef::None),
        Some(path) => path.clone(),
    };

    path.push("HEAD");
    let branch = match std::fs::read(path) {
        Err(..) => return Ok(ValRef::None),
        Ok(contents) => match std::str::from_utf8(&contents[..]) {
            Err(..) => return Ok(ValRef::None),
            Ok(contents) => contents
                .strip_prefix("ref: refs/heads/")
                .and_then(|x| x.strip_suffix("\n"))
                .map(|x| Rc::new(x.to_string())),
        },
    };

    match branch {
        Some(branch) => Ok(ValRef::String(branch)),
        None => Ok(ValRef::None),
    }
}

pub fn init(scope: &Rc<RefCell<Scope>>) {
    let ctx = Rc::new(RefCell::new(GitCtx::new()));

    macro_rules! put {
        ($name: expr, $func: expr) => {
            let c = ctx.clone();
            scope
                .borrow_mut()
                .put_lazy($name, Rc::new(move |_, _| $func(&c)));
        };
    }

    put!("has-git?", has_git);
    put!("git-dir", git_dir);
    put!("git-branch", git_branch);
}
