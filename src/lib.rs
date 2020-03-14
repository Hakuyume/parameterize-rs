mod macros;

use std::cell::RefCell;
use std::fmt::{Arguments, Debug};
use std::panic;
use std::sync::mpsc;
use std::thread;

thread_local! {
    static OUTPUT: RefCell<Output> = RefCell::new(Output::Default);
}

pub fn parameterize<I, F>(params: I, f: F)
where
    I: IntoIterator,
    I::Item: 'static + Debug + Send,
    F: 'static + Copy + Fn(I::Item) + Send,
{
    panic::set_hook(Box::new(|panic_info| {
        __print_fmt(format_args!("{}", panic_info))
    }));

    let th = thread::current();
    let name = th.name().unwrap_or_default();

    let tests = params
        .into_iter()
        .map(|param| {
            let name = format!("{} ({:#?})", name, param);
            let (tx, rx) = mpsc::channel();
            let handle = thread::Builder::new()
                .name(name.clone())
                .spawn(move || {
                    OUTPUT.with(|output| output.replace(Output::Captured(tx)));
                    f(param);
                })
                .unwrap();
            (name, rx, handle)
        })
        .collect::<Vec<_>>();
    let tests = tests
        .into_iter()
        .map(|(name, rx, handle)| {
            let is_ok = handle.join().is_ok();
            let output = rx.try_iter().collect::<String>();
            (name, is_ok, output)
        })
        .collect::<Vec<_>>();

    for (name, is_ok, _) in &tests {
        println!("test {} ... {}", name, status(*is_ok));
    }
    for (name, is_ok, output) in &tests {
        if !*is_ok {
            println!();
            println!("---- {} stdout ----", name);
            println!("{}", output);
        }
    }
    let failed = tests.iter().filter(|(_, is_ok, _)| !is_ok).count();
    if failed != 0 {
        println!();
        panic!("{} of {} tests failed", failed, tests.len());
    }
}

pub fn __print_fmt(fmt: Arguments) {
    OUTPUT.with(|output| match &*output.borrow() {
        Output::Default => std::print!("{}", fmt),
        Output::Captured(tx) => tx.send(format!("{}", fmt)).unwrap(),
    })
}

enum Output {
    Default,
    Captured(mpsc::Sender<String>),
}

#[cfg(feature = "termion")]
fn status(is_ok: bool) -> String {
    let (status, color) = if is_ok {
        ("ok", termion::color::Green.fg_str())
    } else {
        ("FAILED", termion::color::Red.fg_str())
    };
    if termion::is_tty(&std::io::stdout()) {
        format!("{}{}{}", color, status, termion::color::Reset.fg_str())
    } else {
        status.to_owned()
    }
}

#[cfg(not(feature = "termion"))]
fn status(is_ok: bool) -> &'static str {
    if is_ok {
        "ok"
    } else {
        "FAILED"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dbg, println};

    #[test]
    fn test_ok() {
        parameterize(0..10, |i| println!("{}", i));
    }

    #[test]
    #[should_panic]
    fn test_failed() {
        parameterize(0..10, |i| {
            assert_eq!(dbg!(i) % 3, 0);
        });
    }
}
