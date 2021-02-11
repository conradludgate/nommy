#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/single.rs");
    t.pass("tests/multiple.rs");
    t.pass("tests/enum.rs");
    t.pass("tests/mega.rs");
    t.pass("tests/http.rs");
    t.pass("tests/json.rs");
}
