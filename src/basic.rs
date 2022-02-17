use super::state::State;
use dirs;
use glisp::eval::{Scope, ValRef};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use whoami;
use terminal_size;

fn username(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    Ok(ValRef::String(Rc::new(whoami::username())))
}

fn host(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    Ok(ValRef::String(Rc::new(whoami::devicename())))
}

fn replace_home_path(path: String) -> String {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return path,
    };

    match path.strip_prefix(home.to_string_lossy().as_ref()) {
        Some(stripped) => format!("~{}", stripped),
        None => path,
    }
}

fn cwd(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    let wd = match env::current_dir() {
        Ok(wd) => replace_home_path(wd.to_string_lossy().to_string()),
        Err(..) => "".to_string(),
    };

    Ok(ValRef::String(Rc::new(replace_home_path(wd))))
}

#[cfg(unix)]
fn term_size() -> (i32, i32) {
    use terminal_size::{Width, Height};
    // We use stderr (FD 2) because the shell messes with stdin and stdout
    match terminal_size::terminal_size_using_fd(2) {
        Some((Width(w), Height(h))) => (w as i32, h as i32),
        None => (80, 60),
    }
}

#[cfg(not(unix))]
fn term_size() -> (i32, i32) {
    (80, 60)
}

fn term_width(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    let (w, _) = term_size();
    Ok(ValRef::Number(w))
}

fn term_height(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    let (_, h) = term_size();
    Ok(ValRef::Number(h))
}

fn getenv(args: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    if args.len() != 1 {
        return Err("'getenv' requires 1 argument".to_string());
    }

    let key = match &args[0] {
        ValRef::String(s) => s,
        _ => return Err("'getenv' requires a string argument".to_string()),
    };

    let val = match env::var(key.as_ref()) {
        Ok(val) => val,
        Err(err) => return Err(format!("'getenv' failed with key '{}': {}", key, err)),
    };

    Ok(ValRef::String(Rc::new(val)))
}

pub fn init(scope: &Rc<RefCell<Scope>>, state: &Rc<State>) {
    scope
        .borrow_mut()
        .put("exit-code", ValRef::Number(state.exit_code as i32));
    scope
        .borrow_mut()
        .put("space", ValRef::String(Rc::new(" ".to_string())));
    scope.borrow_mut().put_lazy("username", Rc::new(username));
    scope.borrow_mut().put_lazy("host", Rc::new(host));
    scope.borrow_mut().put_lazy("cwd", Rc::new(cwd));
    scope.borrow_mut().put_lazy("term-width", Rc::new(term_width));
    scope.borrow_mut().put_lazy("term-height", Rc::new(term_height));
    scope.borrow_mut().put_func("getenv", Rc::new(getenv));
}
