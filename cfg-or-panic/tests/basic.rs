#![allow(dead_code)]

use cfg_or_panic::cfg_or_panic;

#[cfg_or_panic(not_set)]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct Foo;

#[cfg_or_panic(not_set)]
impl Foo {
    const A: Self = Self::foo();

    const fn foo() -> Self {
        Self
    }

    fn bar(&self) {
        println!("bar")
    }
}

#[cfg_or_panic(not_set)]
mod inner {
    fn sub(a: i32, b: i32) -> i32 {
        a - b
    }

    struct Bar;

    impl Bar {
        fn foo(&self) {
            println!("Foo")
        }
    }
}

#[test]
fn test() {
    Foo.bar()
}
