use cfg_or_panic::cfg_or_panic;

#[cfg_or_panic(foo)]
#[panic_return = "std::iter::Empty<_>"]
fn add_one(iter: impl Iterator<Item = i32>) -> impl Iterator<Item = i32> {
    iter.map(|x| x + 1)
}

#[test]
#[cfg_attr(not(foo), should_panic)]
fn test() {
    assert_eq!(add_one(1..=3).collect::<Vec<_>>(), vec![2, 3, 4]);
}
