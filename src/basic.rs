use super::state::State;
use super::sys;
use dirs;
use osyris::eval::{Scope, ValRef};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;

fn username(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    Ok(ValRef::String(Rc::new(sys::username())))
}

fn host(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    Ok(ValRef::String(Rc::new(sys::hostname())))
}

fn login_name(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    Ok(ValRef::String(Rc::new(sys::login_name())))
}

fn is_remote(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    match env::var("SSH_CLIENT") {
        Ok(_) => Ok(ValRef::Number(1)),
        Err(_) => Ok(ValRef::Number(0)),
    }
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

fn term_width(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    let (w, _) = sys::term_size();
    Ok(ValRef::Number(w))
}

fn term_height(_: Vec<ValRef>, _: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    let (_, h) = sys::term_size();
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
    let mut s = scope.borrow_mut();
    s.put("exit-code", ValRef::Number(state.exit_code as i32));
    s.put("space", ValRef::String(Rc::new(" ".to_string())));
    s.put_lazy("username", Rc::new(username));
    s.put_lazy("host", Rc::new(host));
    s.put_lazy("login-name", Rc::new(login_name));
    s.put_lazy("is-remote?", Rc::new(is_remote));
    s.put_lazy("cwd", Rc::new(cwd));
    s.put_lazy("term-width", Rc::new(term_width));
    s.put_lazy("term-height", Rc::new(term_height));
    s.put_func("getenv", Rc::new(getenv));
}
