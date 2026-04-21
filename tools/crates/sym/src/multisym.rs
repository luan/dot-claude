use std::io::{self, BufRead};

use anyhow::{Result, bail};

pub fn collect_symbols(args: &[String], use_stdin: bool) -> Result<Vec<String>> {
    let stdin = if use_stdin {
        let mut lines = Vec::new();
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        let mut buf = String::new();
        loop {
            buf.clear();
            let read = lock.read_line(&mut buf)?;
            if read == 0 {
                break;
            }
            lines.push(buf.trim_end_matches(['\r', '\n']).to_string());
        }
        Some(lines.join("\n"))
    } else {
        None
    };
    collect_symbols_from(args, use_stdin, stdin.as_deref())
}

pub fn collect_symbols_from(args: &[String], use_stdin: bool, stdin: Option<&str>) -> Result<Vec<String>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();

    let mut add = |value: &str| {
        let value = value.trim();
        if value.is_empty() || value.starts_with('#') || seen.contains(value) {
            return;
        }
        seen.insert(value.to_string());
        out.push(value.to_string());
    };

    for arg in args {
        add(arg);
    }
    if use_stdin {
        if let Some(stdin) = stdin {
            for line in stdin.lines() {
                add(line);
            }
        }
    }
    if out.is_empty() {
        bail!("no symbol names provided (positional args or --stdin)");
    }
    Ok(out)
}

pub fn multi_symbol_header(name: &str, first: bool) -> String {
    if first {
        String::new()
    } else {
        format!("\n═══ {name} ═══\n")
    }
}
