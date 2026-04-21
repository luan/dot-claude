use sym::version;

#[test]
fn version_string_is_non_empty() {
    let value = version::display_version();
    assert!(value.starts_with("sym "));
    assert!(value.len() > 4);
}
