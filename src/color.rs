use super::state::State;
use glisp::eval::{ValRef, Scope};
use std::rc::Rc;
use std::cell::RefCell;

const BLACK: &'static str = "\x1b[30;0m";
const RED: &'static str = "\x1b[31;0m";
const GREEN: &'static str = "\x1b[32;0m";
const YELLOW: &'static str = "\x1b[33;0m";
const BLUE: &'static str = "\x1b[34;0m";
const MAGENTA: &'static str = "\x1b[35;0m";
const CYAN: &'static str = "\x1b[36;0m";
const WHITE: &'static str = "\x1b[37;0m";
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
            stack: vec!(RESET),
        }
    }
}

fn push_color(ctx: &Rc<RefCell<ColorCtx>>, col: &'static str) -> Result<ValRef, String> {
    ctx.borrow_mut().stack.push(col);
    Ok(ValRef::String(Rc::new(format!("{}{}", ctx.borrow().state.shell.escape_start(), col))))
}

fn pop_color(ctx: &Rc<RefCell<ColorCtx>>) -> Result<ValRef, String> {
    ctx.borrow_mut().stack.pop();
    let s = match ctx.borrow_mut().stack.last() {
        Some(s) => *s,
        None => return Err("The color stack should never be empty!".to_string()),
    };
    Ok(ValRef::String(Rc::new(format!("{}{}", ctx.borrow().state.shell.escape_end(), s))))
}

fn color(ctx: &Rc<RefCell<ColorCtx>>, col: &'static str, args: Vec<ValRef>) -> Result<ValRef, String> {
    let mut ret: Vec<ValRef> = Vec::new();

    {
        let c = ctx.clone();
        ret.push(ValRef::Func(Rc::new(move |_, _| push_color(&c, col))));
    }

    ret.push(ValRef::List(Rc::new(args)));

    {
        let c = ctx.clone();
        ret.push(ValRef::Func(Rc::new(move |_, _| pop_color(&c))));
    }

    Ok(ValRef::List(Rc::new(ret)))
}

pub fn init(scope: &Rc<RefCell<Scope>>, state: &Rc<State>) {
    let ctx = Rc::new(RefCell::new(ColorCtx::new(state.clone())));

    macro_rules! put{($name: expr, $color: expr) => {
        {
            let c = ctx.clone();
            scope.borrow_mut().put_func($name, Rc::new(move |a, _| color(&c, $color, a)));
        }
    }}

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
}
