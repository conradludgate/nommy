#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/single.rs");
    t.pass("tests/multiple.rs");
}
