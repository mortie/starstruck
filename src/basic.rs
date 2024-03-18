use super::state::State;
use super::sys;
use dirs;
use osyris::bstring::BString;
use osyris::eval::{FuncResult, Scope, StackTrace, ValRef};
use std::env;
use std::rc::Rc;

fn username(_: Vec<ValRef>, scope: Scope) -> FuncResult {
    Ok((
        ValRef::String(Rc::new(BString::from_string(sys::username()))),
        scope,
    ))
}

fn host(_: Vec<ValRef>, scope: Scope) -> FuncResult {
    Ok((
        ValRef::String(Rc::new(BString::from_string(sys::hostname()))),
        scope,
    ))
}

fn login_name(_: Vec<ValRef>, scope: Scope) -> FuncResult {
    Ok((
        ValRef::String(Rc::new(BString::from_string(sys::login_name()))),
        scope,
    ))
}

fn is_remote(_: Vec<ValRef>, scope: Scope) -> FuncResult {
    match env::var("SSH_CLIENT") {
        Ok(_) => Ok((ValRef::Bool(true), scope)),
        Err(_) => Ok((ValRef::Bool(false), scope)),
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

fn cwd(_: Vec<ValRef>, scope: Scope) -> FuncResult {
    let wd = match env::current_dir() {
        Ok(wd) => replace_home_path(BString::from_os_str(wd.as_os_str())),
        Err(..) => BString::from_str(""),
    };

    Ok((ValRef::String(Rc::new(replace_home_path(wd))), scope))
}

fn term_width(_: Vec<ValRef>, scope: Scope) -> FuncResult {
    let (w, _) = sys::term_size();
    Ok((ValRef::Number(w as f64), scope))
}

fn term_height(_: Vec<ValRef>, stack: Scope) -> FuncResult {
    let (_, h) = sys::term_size();
    Ok((ValRef::Number(h as f64), stack))
}

fn getenv(args: Vec<ValRef>, stack: Scope) -> FuncResult {
    if args.len() != 1 {
        return Err(StackTrace::from_str("'getenv' requires 1 argument"));
    }

    let key = match &args[0] {
        ValRef::String(s) => s,
        _ => return Err(StackTrace::from_str("'getenv' requires a string argument")),
    };

    let val = match env::var(key.to_os_str()) {
        Ok(val) => BString::from_string(val),
        Err(err) => {
            return Err(StackTrace::from_string(format!(
                "'getenv' failed with key '{}': {}",
                key, err
            )))
        }
    };

    Ok((ValRef::String(Rc::new(val)), stack))
}

pub fn init(mut s: Scope, state: &Rc<State>) -> Scope {
    s = s.put("exit-code", ValRef::Number(state.exit_code as f64));
    s = s.put("space", ValRef::String(Rc::new(BString::from_str(" "))));
    s = s.put_lazy("username", Rc::new(username));
    s = s.put_lazy("host", Rc::new(host));
    s = s.put_lazy("login-name", Rc::new(login_name));
    s = s.put_lazy("is-remote?", Rc::new(is_remote));
    s = s.put_lazy("cwd", Rc::new(cwd));
    s = s.put_lazy("term-width", Rc::new(term_width));
    s = s.put_lazy("term-height", Rc::new(term_height));
    s = s.put_func("getenv", Rc::new(getenv));
    s
}
