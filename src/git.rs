use osyris::bstring::BString;
use osyris::eval::{Scope, StackTrace, ValRef};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
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

    fn find_gitdir_from_file(&mut self, mut path: PathBuf) -> Option<PathBuf> {
        let f = match fs::File::open(&path) {
            Err(..) => return None,
            Ok(f) => f,
        };

        let content = match BufReader::new(f).lines().next() {
            None => return None,
            Some(content) => match content {
                Err(..) => return None,
                Ok(content) => content,
            },
        };

        if let Some(p) = content.strip_prefix("gitdir: ") {
            path.pop(); // Remove the .git component
            return Some(path.join(p));
        }

        None
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
                    } else if meta.is_file() {
                        self.has_searched_gitdir = true;
                        self.gitdir = self.find_gitdir_from_file(path);
                        return self.gitdir.is_some();
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

fn has_git(ctx: &Rc<RefCell<GitCtx>>) -> Result<ValRef, StackTrace> {
    if ctx.borrow_mut().find_gitdir() {
        Ok(ValRef::Bool(true))
    } else {
        Ok(ValRef::Bool(false))
    }
}

fn git_dir(ctx: &Rc<RefCell<GitCtx>>) -> Result<ValRef, StackTrace> {
    ctx.borrow_mut().find_gitdir();
    match &ctx.borrow().gitdir {
        Some(s) => Ok(ValRef::String(Rc::new(BString::from_os_str(s.as_os_str())))),
        None => Ok(ValRef::None),
    }
}

fn git_branch(ctx: &Rc<RefCell<GitCtx>>) -> Result<ValRef, StackTrace> {
    ctx.borrow_mut().find_gitdir();
    let mut path = match &ctx.borrow().gitdir {
        None => return Ok(ValRef::None),
        Some(path) => path.clone(),
    };

    path.push("HEAD");
    let f = match fs::File::open(&path) {
        Err(..) => return Ok(ValRef::None),
        Ok(f) => f,
    };

    let content = match BufReader::new(f).split(b'\n').next() {
        None => return Ok(ValRef::None),
        Some(content) => match content {
            Err(..) => return Ok(ValRef::None),
            Ok(content) => content,
        },
    };

    let branch = if let Some(branch) = content.strip_prefix(b"ref: refs/heads/") {
        branch
    } else {
        &content[..8]
    };

    return Ok(ValRef::String(Rc::new(BString::from_bytes(branch))));
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
