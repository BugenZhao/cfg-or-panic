use cfg_or_panic::cfg_or_panic;

#[cfg_or_panic(not_set)]
#[test]
fn test() {
    println!("Hello, world!");
}
