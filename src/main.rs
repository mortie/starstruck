mod basic;
mod color;
mod git;
mod state;
mod sys;

use dirs;
use eval::{Scope, ValRef};
use osyris::{eval, iolib, parse, stdlib};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::rc::Rc;

pub struct UncountedString {
    pub s: String,
}

struct Printer {
    column: i32,
    row: i32,
}

impl Printer {
    fn print(&mut self, s: &String) {
        for ch in s.chars() {
            if ch == '\n' {
                self.column = 1;
                self.row += 1;
            } else {
                self.column += 1;
            }
        }

        self.print_uncounted(s);
    }

    fn print_uncounted(&self, s: &String) {
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
        ValRef::Number(num) => printer.borrow_mut().print(&format!("{}", num)),
        ValRef::Bool(b) => printer.borrow_mut().print(&format!("{}", b)),
        ValRef::Map(..) => (),
        ValRef::List(lst) => {
            for item in lst.as_ref() {
                print_ps1(&printer, item.clone(), scope)?;
            }
        }
        ValRef::String(s) => printer.borrow_mut().print(s.as_ref()),
        ValRef::Func(func) => {
            let args = Vec::new();
            print_ps1(printer, func(args, scope)?, scope)?
        }
        ValRef::Quote(exprs) => {
            print_ps1(printer, eval::eval_call(exprs.as_ref(), scope)?, scope)?;
        }
        ValRef::Native(n) => {
            if let Some(us) = n.as_ref().downcast_ref::<UncountedString>() {
                printer.borrow().print_uncounted(&us.s);
            }
        }
        ValRef::Port(..) => (),
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

fn find_config_path() -> Option<PathBuf> {
    if let Some(mut dir) = dirs::home_dir() {
        dir.push(".config");

        let mut path = dir.clone();
        path.push("starstruck");
        path.push("main.lsp");
        if Path::exists(&path) {
            return Some(path);
        }

        let mut path = dir;
        path.push("starstruck.lsp");
        if Path::exists(&path) {
            return Some(path);
        }
    }

    if let Some(dir) = dirs::config_dir() {
        let mut path = dir.clone();
        path.push("starstruck");
        path.push("main.lsp");
        if Path::exists(&path) {
            return Some(path);
        }

        let mut path = dir;
        path.push("starstruck.lsp");
        if Path::exists(&path) {
            return Some(path);
        }
    }

    None
}

fn usage(argv0: String) {
    println!("Usage: {} [options]", argv0);
    println!();
    println!("Options:");
    println!("  -h, --help: Show this help text");
    println!("  -c <path>:  Config file path");
    println!("  -e <code>:  Set the exit code of the previous command");
    println!("  --bash:     Set the shell to bash");
    println!("  --zsh:      Set the shell to zsh");
}

fn main() {
    let mut config_path = find_config_path();

    let mut state = state::State {
        exit_code: 0,
        shell: state::Shell::None,
    };

    let mut args = env::args();
    let argv0 = args.next().unwrap();

    while let Some(arg) = args.next() {
        if arg == "-h" || arg == "--help" {
            usage(argv0);
            return;
        } else if arg == "-c" {
            config_path = match args.next() {
                Some(p) => Some(PathBuf::from(p)),
                None => {
                    eprintln!("Option 'c' requires an argument");
                    process::exit(1);
                }
            };
        } else if arg == "-e" {
            state.exit_code = match args.next() {
                Some(s) => match str::parse::<u8>(&s) {
                    Ok(code) => code,
                    Err(err) => {
                        eprintln!("Invalid exit code '{}': {}", s, err);
                        process::exit(1);
                    }
                },
                None => {
                    eprintln!("Option 'e' requires an argument");
                    process::exit(1);
                }
            }
        } else if arg == "--bash" {
            state.shell = state::Shell::Bash;
        } else if arg == "--zsh" {
            state.shell = state::Shell::Zsh;
        } else {
            eprintln!("Unexpected argument: {}", arg);
            usage(argv0);
            process::exit(1);
        }
    }

    let config_path = match config_path {
        Some(p) => p,
        None => {
            eprintln!("No config file found. Specify a config file path with '-c',");
            eprintln!("or put a config file in <CONFIG_HOME>/starstruck.lsp.");
            process::exit(1);
        }
    };

    let file_string = match fs::read_to_string(&config_path) {
        Ok(string) => string,
        Err(err) => {
            eprintln!("{:?}: {}", config_path, err);
            process::exit(1);
        }
    };

    let state = Rc::new(state);
    let printer = Rc::new(RefCell::new(Printer { column: 1, row: 1 }));

    let scope = Rc::new(RefCell::new(eval::Scope::new()));
    stdlib::init(&scope);
    iolib::init(&scope);
    basic::init(&scope, &state);
    color::init(&scope, &state);
    git::init(&scope);

    {
        let s = printer.clone();
        scope.borrow_mut().put_lazy(
            "column",
            Rc::new(move |_, _| Ok(ValRef::Number(s.borrow().column as f64))),
        );
        let s = printer.clone();
        scope.borrow_mut().put_lazy(
            "row",
            Rc::new(move |_, _| Ok(ValRef::Number(s.borrow().row as f64))),
        );
    }

    let mut reader = parse::Reader::new(&file_string.as_bytes());
    let retval = match execute_file(&mut reader, &scope) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    match print_ps1(&printer, retval, &scope) {
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
        _ => (),
    };
}
