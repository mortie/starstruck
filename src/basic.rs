use super::state::State;
use dirs;
use glisp::eval::{Scope, ValRef};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;
use whoami;

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
    scope.borrow_mut().put_func("getenv", Rc::new(getenv));
}
