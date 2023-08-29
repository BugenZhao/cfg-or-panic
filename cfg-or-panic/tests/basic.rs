use cfg_or_panic::cfg_or_panic;

#[cfg_or_panic(not_set)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct Foo;

#[cfg_or_panic(not_set)]
impl Foo {
    fn bar(&self) {}
}

#[cfg_or_panic(not_set)]
#[test]
fn test() {
    println!("Hello, world!");
}
