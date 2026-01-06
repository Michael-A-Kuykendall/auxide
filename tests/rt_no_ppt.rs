use std::fs;
use std::path::Path;

/// Ensure RT module does not call the non-RT `assert_invariant` which acquires a Mutex.
#[test]
fn rt_does_not_call_assert_invariant() {
    let rt_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("rt.rs");
    let src = fs::read_to_string(rt_path).expect("failed to read rt.rs");
    assert!(
        !src.contains("assert_invariant("),
        "RT paths must not call assert_invariant (acquires Mutex). Use `assert_invariant_rt_safe` or avoid invariant logging in RT."
    );
}
