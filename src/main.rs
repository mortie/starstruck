mod basic;
mod color;
mod git;
mod state;

use eval::{Scope, ValRef};
use glisp::{eval, parse, stdlib};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::rc::Rc;

struct Printer {
    column: i32,
    row: i32,
}

impl Printer {
    fn print(&mut self, s: String) {
        for ch in s.chars() {
            if ch == '\n' {
                self.column = 1;
                self.row += 1;
            } else {
                self.column += 1;
            }
        }

        print!("{}", s)
    }
}

fn print_ps1(
    printer: &Rc<RefCell<Printer>>,
    val: ValRef,
    scope: &Rc<RefCell<Scope>>,
) -> Result<(), String> {
    match val {
        ValRef::None => (),
        ValRef::Lazy(..) => (),
        ValRef::ProtectedLazy(..) => (),
        ValRef::Number(num) => printer.borrow_mut().print(format!("{}", num)),
        ValRef::Map(..) => (),
        ValRef::List(lst) => {
            for item in lst.as_ref() {
                print_ps1(&printer, item.clone(), scope)?;
            }
        }
        ValRef::String(s) => printer.borrow_mut().print(s.as_ref().clone()),
        ValRef::Func(func) => {
            let args = Vec::new();
            print_ps1(printer, func(args, scope)?, scope)?
        }
        ValRef::Quote(exprs) => {
            print_ps1(printer, eval::eval_call(exprs.as_ref(), scope)?, scope)?;
        }
    }

    Ok(())
}

fn execute_file(reader: &mut parse::Reader, scope: &Rc<RefCell<Scope>>) -> Result<ValRef, String> {
    let mut retval = ValRef::None;
    loop {
        let expr = match parse::parse(reader) {
            Ok(expr) => match expr {
                Some(expr) => expr,
                None => return Ok(retval),
            },
            Err(err) => {
                return Err(format!(
                    "Parse error: {}:{}: {}",
                    err.line, err.col, err.msg
                ))
            }
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

    let printer = Rc::new(RefCell::new(Printer { column: 1, row: 1 }));

    let scope = Rc::new(RefCell::new(eval::Scope::new(None)));
    stdlib::init(&scope);
    basic::init(&scope, &state);
    color::init(&scope, &state);
    git::init(&scope);

    {
        let s = printer.clone();
        scope.borrow_mut().put_lazy(
            "column",
            Rc::new(move |_, _| Ok(ValRef::Number(s.borrow().column))),
        );
        let s = printer.clone();
        scope.borrow_mut().put_lazy(
            "row",
            Rc::new(move |_, _| Ok(ValRef::Number(s.borrow().row))),
        );
    }

    let mut reader = parse::Reader::new(&file_string.as_bytes());
    let retval = match execute_file(&mut reader, &scope) {
        Ok(val) => val,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    match print_ps1(&printer, retval, &scope) {
        Err(err) => {
            print!("Error: {}", err);
            return;
        }
        _ => (),
    };
}
