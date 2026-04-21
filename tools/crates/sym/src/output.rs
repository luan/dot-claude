use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use serde::Serialize;

static JSON_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_json_enabled(enabled: bool) {
    JSON_ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn json_enabled() -> bool {
    JSON_ENABLED.load(Ordering::Relaxed)
}

pub fn write_json<T: Serialize>(data: &T) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(
        &mut handle,
        &Envelope {
            version: "0.1",
            results: data,
        },
    )?;
    handle.write_all(b"\n")?;
    Ok(())
}

pub fn render<T: Serialize>(data: &T, meta: &[(&str, String)], content: &str) -> Result<()> {
    if json_enabled() {
        return write_json(data);
    }
    write_frontmatter(meta, content)
}

pub fn write_frontmatter(meta: &[(&str, String)], content: &str) -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(b"---\n")?;
    for (key, value) in meta {
        handle.write_all(key.as_bytes())?;
        handle.write_all(b": ")?;
        handle.write_all(value.as_bytes())?;
        handle.write_all(b"\n")?;
    }
    handle.write_all(b"---\n")?;
    if !content.is_empty() {
        handle.write_all(content.as_bytes())?;
        if !content.ends_with('\n') {
            handle.write_all(b"\n")?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct Envelope<'a, T> {
    version: &'static str,
    results: &'a T,
}
