use anyhow::Result;

use sym::multisym::{collect_symbols_from, multi_symbol_header};

#[test]
fn collect_symbols_merges_args_and_stdin_in_order() -> Result<()> {
    let symbols = collect_symbols_from(
        &["Cache".into(), "Reader".into()],
        true,
        Some("Reader\n# comment\nHandle\n\nCache\nTrace\n"),
    )?;
    assert_eq!(symbols, vec!["Cache", "Reader", "Handle", "Trace"]);
    Ok(())
}

#[test]
fn collect_symbols_requires_at_least_one_symbol() {
    let error = collect_symbols_from(&[], true, Some("\n# nothing\n")).unwrap_err();
    assert!(error.to_string().contains("no symbol names provided"));
}

#[test]
fn multi_symbol_header_is_only_emitted_after_first_symbol() {
    assert_eq!(multi_symbol_header("Cache", true), "");
    assert_eq!(multi_symbol_header("Reader", false), "\n═══ Reader ═══\n");
}
