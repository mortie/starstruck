mod state;
mod basic;
mod color;

use glisp::{parse, eval, stdlib};
use eval::{ValRef, Scope};
use std::env;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

fn print_ps1(val: ValRef, scope: &Rc<RefCell<Scope>>) -> Result<(), String> {
    match val {
        ValRef::None => (),
        ValRef::Lazy(..) => (),
        ValRef::ProtectedLazy(..) => (),
        ValRef::Number(num) => print!("{}", num),
        ValRef::Map(..) => (),
        ValRef::List(lst) => {
            for item in lst.as_ref() {
                print_ps1(item.clone(), scope)?;
            }
        }
        ValRef::String(s) => print!("{}", s),
        ValRef::Func(func) => {
            let args = Vec::new();
            print_ps1(func(args, scope)?, scope)?
        }
        ValRef::Quote(exprs) => {
            let mut retval = ValRef::None;
            for expr in exprs.as_ref() {
                retval = eval::eval(expr, scope)?;
            }

            print_ps1(retval, scope)?;
        }
    }

    Ok(())
}

fn execute_file(reader: &mut parse::Reader, scope: &Rc<RefCell<Scope>>) -> Result<ValRef, String>{
    let mut retval = ValRef::None;
    loop {
        let expr = match parse::parse(reader) {
            Ok(expr) => match expr {
                Some(expr) => expr,
                None => return Ok(retval),
            }
            Err(err) => return Err(format!("Parse error: {}:{}: {}", err.line, err.col, err.msg)),
        };

        match eval::eval(&expr, &scope) {
            Err(err) => return Err(format!("Error: {}", err)),
            Ok(val) => retval = val,
        }
    }
}

fn main() {
    let mut args = env::args();
    args.next().unwrap();

    let path = match args.next() {
        Some(path) => path,
        None => {
            println!("Need argument");
            return;
        }
    };

    let file_string = match fs::read_to_string(&path) {
        Ok(string) => string,
        Err(err) => {
            println!("{}: {}", path, err);
            return;
        }
    };

    let state = Rc::new(state::State {
        exit_code: 0,
        shell: state::Shell::None,
    });

    let scope = Rc::new(RefCell::new(eval::Scope::new(None)));
    stdlib::init(&scope);
    basic::init(&scope);
    color::init(&scope, &state);

    let mut reader = parse::Reader::new(&file_string.as_bytes());
    let retval = match execute_file(&mut reader, &scope) {
        Ok(val) => val,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    match print_ps1(retval, &scope) {
        Err(err) => {
            print!("Error: {}", err);
            return;
        }
        _ => ()
    };
}
