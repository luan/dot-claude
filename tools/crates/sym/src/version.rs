pub fn display_version() -> String {
    format!("sym {}", env!("CARGO_PKG_VERSION"))
}
