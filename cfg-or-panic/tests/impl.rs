use std::ops::Add;

use cfg_or_panic::cfg_or_panic;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Int(i32);

#[cfg_or_panic(foo)]
impl Int {
    fn my_add(&self, b: i32) -> Int {
        Int(self.0 + b)
    }
}

#[cfg_or_panic(foo)]
impl Add<i32> for Int {
    type Output = Int;

    fn add(self, rhs: i32) -> Self::Output {
        self.my_add(rhs)
    }
}

#[test]
#[cfg_attr(not(foo), should_panic)]
fn test_fn() {
    assert_eq!(Int(1).my_add(2), Int(3));
}

#[test]
#[cfg_attr(not(foo), should_panic)]
fn test_trait_fn() {
    assert_eq!(Int(1) + 2, Int(3));
}
