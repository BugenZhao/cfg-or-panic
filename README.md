# `#[cfg_or_panic(..)]`

Keep the function body under `#[cfg(..)]`, or replace it with `unimplemented!()` under `#[cfg(not(..))]`.

## Example

```rust
use cfg_or_panic::cfg_or_panic;

#[cfg_or_panic(feature = "foo")]
fn foo() -> i32 {
    42
}

#[test]
#[cfg_attr(not(feature = "foo"), should_panic)]
fn test() {
    assert_eq!(foo(), 42);
}
```
