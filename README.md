# parameterize

Helper for parameterized tests in Rust

## Example
```rust
use parameterize::{dbg, parameterize, println};

#[test]
fn test_ok() {
    parameterize(0..10, |i| println!("{}", i));
}

#[test]
fn test_failed() {
    parameterize(0..5, |i| {
        assert!(dbg!(i % 3) != 2);
    });
}
```

```
running 2 tests
test test_failed ... FAILED
test test_ok ... ok

failures:

---- test_failed stdout ----
test test_failed (0) ... ok
test test_failed (1) ... ok
test test_failed (2) ... FAILED
test test_failed (3) ... ok
test test_failed (4) ... ok
---- test_failed (2) stdout ----
[tests/demo.rs:11] i % 3 = 2
panicked at 'assertion failed: dbg!(i % 3) != 2', tests/demo.rs:11:9

panicked at '1 of 5 tests failed', <::std::macros::panic macros>:5:6

failures:
    test_failed

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```
