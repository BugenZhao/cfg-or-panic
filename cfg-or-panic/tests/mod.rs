use cfg_or_panic::cfg_or_panic;

#[cfg_or_panic(foo)]
mod inner {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

#[test]
#[cfg_attr(not(foo), should_panic)]
fn test() {
    assert_eq!(inner::add(1, 2), 3);
}
