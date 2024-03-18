use super::state::State;
use super::UncountedString;
use osyris::eval::{Scope, StackTrace, ValRef};
use std::cell::RefCell;
use std::rc::Rc;

const BLACK: &'static str = "\x1b[30m";
const RED: &'static str = "\x1b[31m";
const GREEN: &'static str = "\x1b[32m";
const YELLOW: &'static str = "\x1b[33m";
const BLUE: &'static str = "\x1b[34m";
const MAGENTA: &'static str = "\x1b[35m";
const CYAN: &'static str = "\x1b[36m";
const WHITE: &'static str = "\x1b[37m";
const RESET: &'static str = "\x1b[0m";

const BOLD_BLACK: &'static str = "\x1b[30;1m";
const BOLD_RED: &'static str = "\x1b[31;1m";
const BOLD_GREEN: &'static str = "\x1b[32;1m";
const BOLD_YELLOW: &'static str = "\x1b[33;1m";
const BOLD_BLUE: &'static str = "\x1b[34;1m";
const BOLD_MAGENTA: &'static str = "\x1b[35;1m";
const BOLD_CYAN: &'static str = "\x1b[36;1m";
const BOLD_WHITE: &'static str = "\x1b[37;1m";

struct ColorCtx {
    state: Rc<State>,
    stack: Vec<&'static str>,
}

impl ColorCtx {
    fn new(state: Rc<State>) -> Self {
        Self {
            state,
            stack: Vec::new(),
        }
    }
}

fn push_color(ctx: &Rc<RefCell<ColorCtx>>, col: &'static str) -> Result<ValRef, StackTrace> {
    ctx.borrow_mut().stack.push(col);
    let c = ctx.borrow();
    let escape_start = c.state.shell.escape_start();
    let escape_end = c.state.shell.escape_end();
    let s = match c.stack.len() {
        0 => format!("{}{}{}", escape_start, col, escape_end),
        _ => format!("{}{}{}{}", escape_start, RESET, col, escape_end),
    };
    Ok(ValRef::Native(Rc::new(UncountedString { s })))
}

fn pop_color(ctx: &Rc<RefCell<ColorCtx>>) -> Result<ValRef, StackTrace> {
    ctx.borrow_mut().stack.pop();
    let c = ctx.borrow();
    let escape_start = c.state.shell.escape_start();
    let escape_end = c.state.shell.escape_end();
    let s = match c.stack.last() {
        None => format!("{}{}{}", escape_start, RESET, escape_end),
        Some(last) => format!("{}{}{}{}", escape_start, RESET, last, escape_end),
    };

    Ok(ValRef::Native(Rc::new(UncountedString { s })))
}

fn color(
    ctx: &Rc<RefCell<ColorCtx>>,
    col: &'static str,
    args: Vec<ValRef>,
) -> Result<ValRef, StackTrace> {
    let mut ret: Vec<ValRef> = Vec::new();

    {
        let c = ctx.clone();
        ret.push(ValRef::Func(Rc::new(move |_, scope| {
            Ok((push_color(&c, col)?, scope))
        })));
    }

    ret.push(ValRef::List(Rc::new(RefCell::new(args.to_vec()))));

    {
        let c = ctx.clone();
        ret.push(ValRef::Func(Rc::new(move |_, scope| {
            Ok((pop_color(&c)?, scope))
        })));
    }

    Ok(ValRef::List(Rc::new(RefCell::new(ret))))
}

pub fn init(mut scope: Scope, state: &Rc<State>) -> Scope {
    let ctx = Rc::new(RefCell::new(ColorCtx::new(state.clone())));

    macro_rules! put {
        ($name: expr, $color: expr) => {
            let c = ctx.clone();
            scope = scope.put_func(
                $name,
                Rc::new(move |a, scope| Ok((color(&c, $color, a)?, scope))),
            );
        };
    }

    put!("black", BLACK);
    put!("red", RED);
    put!("green", GREEN);
    put!("yellow", YELLOW);
    put!("blue", BLUE);
    put!("magenta", MAGENTA);
    put!("cyan", CYAN);
    put!("white", WHITE);
    put!("reset", RESET);

    put!("bold-black", BOLD_BLACK);
    put!("bold-red", BOLD_RED);
    put!("bold-green", BOLD_GREEN);
    put!("bold-yellow", BOLD_YELLOW);
    put!("bold-blue", BOLD_BLUE);
    put!("bold-magenta", BOLD_MAGENTA);
    put!("bold-cyan", BOLD_CYAN);
    put!("bold-white", BOLD_WHITE);

    scope
}
