mod basic;
mod color;
mod git;
mod state;
mod sys;

use dirs;
use eval::{Scope, StackTrace, ValRef};
use osyris::bstring::BString;
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
    mut val: ValRef,
    mut scope: Scope,
) -> Result<Scope, StackTrace> {
    match val {
        ValRef::None => (),
        ValRef::Lazy(..) => (),
        ValRef::ProtectedLazy(..) => (),
        ValRef::Number(num) => printer.borrow_mut().print(&format!("{}", num)),
        ValRef::Bool(b) => printer.borrow_mut().print(&format!("{}", b)),
        ValRef::Dict(..) => (),
        ValRef::List(lst) => {
            for item in lst.borrow().iter() {
                scope = print_ps1(&printer, item.clone(), scope)?;
            }
        }
        ValRef::String(s) => printer
            .borrow_mut()
            .print(&String::from_utf8_lossy(s.as_bytes()).to_string()),
        ValRef::Native(n) => {
            if let Some(us) = n.as_ref().downcast_ref::<UncountedString>() {
                printer.borrow().print_uncounted(&us.s);
            }
        }
        ValRef::Port(..) => (),
        ValRef::Block(exprs) => {
            for expr in exprs.iter() {
                (val, scope) = eval::eval(expr, scope)?;
                scope = print_ps1(printer, val, scope)?;
            }
        }
        _ => {
            (val, scope) = eval::call(&val, vec![], scope)?;
            scope = print_ps1(printer, val, scope)?;
        }
    };

    Ok(scope)
}

fn execute_file(reader: &mut parse::Reader, mut scope: Scope) -> Result<ValRef, String> {
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

        match eval::eval(&expr, scope) {
            Err(err) => return Err(format!("Error: {}", err)),
            Ok((val, s)) => {
                retval = val;
                scope = s;
            }
        }
    }
}

fn find_config_path_from_base(base: PathBuf) -> Option<PathBuf> {
    let mut path = base.clone();
    path.push("starstruck");
    path.push("main.lsp");
    if Path::exists(&path) {
        return Some(path);
    }

    let mut path = base;
    path.push("starstruck.lsp");
    if Path::exists(&path) {
        return Some(path);
    }

    None
}

fn find_config_path() -> Option<PathBuf> {
    if let Some(dir) = dirs::config_dir() {
        if let Some(path) = find_config_path_from_base(dir) {
            return Some(path);
        }
    }

    if let Some(mut dir) = dirs::home_dir() {
        dir.push(".config");
        if let Some(path) = find_config_path_from_base(dir) {
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
        }

        if arg == "-c" {
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

    let mut scope = eval::Scope::new();
    scope = stdlib::init(scope);
    scope = iolib::init(scope);
    scope = basic::init(scope, &state);
    scope = color::init(scope, &state);
    scope = git::init(scope);

    {
        let s = printer.clone();
        scope = scope.put_lazy(
            "column",
            Rc::new(move |_, scope| Ok((ValRef::Number(s.borrow().column as f64), scope))),
        );
        let s = printer.clone();
        scope = scope.put_lazy(
            "row",
            Rc::new(move |_, scope| Ok((ValRef::Number(s.borrow().row as f64), scope))),
        );
    }

    let mut reader = parse::Reader::new(
        &file_string.as_bytes(),
        BString::from_os_str(config_path.as_os_str()),
    );
    let retval = match execute_file(&mut reader, scope.clone()) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    match print_ps1(&printer, retval, scope) {
        Err(err) => {
            eprintln!("Error: {}", err);
            return;
        }
        _ => (),
    };
}
