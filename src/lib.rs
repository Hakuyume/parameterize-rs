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
            let dbg_param = format!("{:#?}", param);
            let (tx, rx) = mpsc::channel();
            let handle = thread::spawn(move || {
                OUTPUT.with(|output| output.replace(Output::Captured(tx)));
                f(param);
            });
            (dbg_param, rx, handle)
        })
        .collect::<Vec<_>>();
    let tests = tests
        .into_iter()
        .map(|(param, rx, handle)| {
            let is_ok = handle.join().is_ok();
            let output = rx.try_iter().collect::<String>();
            (param, is_ok, output)
        })
        .collect::<Vec<_>>();

    for (param, is_ok, _) in &tests {
        if *is_ok {
            println!("test {} ({}) ... ok", name, param);
        } else {
            println!("test {} ({}) ... FAILED", name, param);
        }
    }
    for (param, is_ok, output) in &tests {
        if !*is_ok {
            println!();
            println!("---- {} ({}) stdout ----", name, param);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dbg, println};

    #[test]
    fn test_ok() {
        parameterize(0..10, |i| println!("{}", i));
    }

    #[test]
    fn test_failed() {
        parameterize(0..10, |i| {
            assert_eq!(dbg!(i) % 3, 0);
        });
    }
}
