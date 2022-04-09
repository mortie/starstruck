use super::state::State;
use super::sys;
use dirs;
use osyris::eval::{Scope, ValRef, StackTrace};
use osyris::bstring::BString;
use std::cell::RefCell;
use std::env;
use std::rc::Rc;

fn username(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    Ok(ValRef::String(Rc::new(BString::from_string(sys::username()))))
}

fn host(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    Ok(ValRef::String(Rc::new(BString::from_string(sys::hostname()))))
}

fn login_name(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    Ok(ValRef::String(Rc::new(BString::from_string(sys::login_name()))))
}

fn is_remote(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    match env::var("SSH_CLIENT") {
        Ok(_) => Ok(ValRef::Bool(true)),
        Err(_) => Ok(ValRef::Bool(false)),
    }
}

fn replace_home_path(path: BString) -> BString {
    let home = match dirs::home_dir() {
        Some(home) => BString::from_os_str(home.as_os_str()),
        None => return path,
    };

    match path.strip_prefix(&home) {
        Some(stripped) => BString::from_string(format!("~{}", String::from_utf8_lossy(stripped))),
        None => path,
    }
}

fn cwd(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    let wd = match env::current_dir() {
        Ok(wd) => replace_home_path(BString::from_os_str(wd.as_os_str())),
        Err(..) => BString::from_str(""),
    };

    Ok(ValRef::String(Rc::new(replace_home_path(wd))))
}

fn term_width(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    let (w, _) = sys::term_size();
    Ok(ValRef::Number(w as f64))
}

fn term_height(_: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    let (_, h) = sys::term_size();
    Ok(ValRef::Number(h as f64))
}

fn getenv(args: &[ValRef], _: &Rc<RefCell<Scope>>) -> Result<ValRef, StackTrace> {
    if args.len() != 1 {
        return Err(StackTrace::from_str("'getenv' requires 1 argument"));
    }

    let key = match &args[0] {
        ValRef::String(s) => s,
        _ => return Err(StackTrace::from_str("'getenv' requires a string argument")),
    };

    let val = match env::var(key.to_os_str()) {
        Ok(val) => BString::from_string(val),
        Err(err) => return Err(StackTrace::from_string(format!("'getenv' failed with key '{}': {}", key, err))),
    };

    Ok(ValRef::String(Rc::new(val)))
}

pub fn init(scope: &Rc<RefCell<Scope>>, state: &Rc<State>) {
    let mut s = scope.borrow_mut();
    s.put("exit-code", ValRef::Number(state.exit_code as f64));
    s.put("space", ValRef::String(Rc::new(BString::from_str(" "))));
    s.put_lazy("username", Rc::new(username));
    s.put_lazy("host", Rc::new(host));
    s.put_lazy("login-name", Rc::new(login_name));
    s.put_lazy("is-remote?", Rc::new(is_remote));
    s.put_lazy("cwd", Rc::new(cwd));
    s.put_lazy("term-width", Rc::new(term_width));
    s.put_lazy("term-height", Rc::new(term_height));
    s.put_func("getenv", Rc::new(getenv));
}
