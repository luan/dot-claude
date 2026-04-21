use std::fs;
use std::path::Path;

use anyhow::Result;
use sym::outline;
use sym::show;

#[test]
fn outline_lists_symbols_in_file_order() -> Result<()> {
    let fixture = Fixture::new()?;

    let symbols = outline::file_outline(fixture.root(), &fixture.root().join("src/main.go"))?;

    assert_eq!(symbols.len(), 3);
    assert_eq!(symbols[0].name, "Server");
    assert_eq!(symbols[1].name, "Start");
    assert_eq!(symbols[1].kind, "method");
    assert_eq!(symbols[2].name, "main");

    Ok(())
}

#[test]
fn show_reads_symbol_source() -> Result<()> {
    let fixture = Fixture::new()?;

    let shown = show::show_symbol(fixture.root(), "Start", 0, false)?;

    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].symbol.name, "Start");
    assert!(shown[0].content.contains("func (s *Server) Start() error"));
    assert!(shown[0].content.contains("fmt.Println(\"starting\")"));

    Ok(())
}

#[test]
fn show_reads_file_ranges_with_context() -> Result<()> {
    let fixture = Fixture::new()?;

    let shown = show::show_file(&fixture.root().join("src/main.go"), Some((8, 8)), 1)?;

    let line_numbers = shown.iter().map(|line| line.line).collect::<Vec<_>>();
    assert_eq!(line_numbers, vec![7, 8, 9]);
    assert!(shown[1].content.contains("fmt.Println(\"starting\")"));

    Ok(())
}

struct Fixture {
    root: tempfile::TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join(".git"))?;
        write(
            root.path(),
            "src/main.go",
            r#"package main

import "fmt"

type Server struct {}

func (s *Server) Start() error {
    fmt.Println("starting")
    return nil
}

func main() {
    srv := &Server{}
    _ = srv.Start()
}
"#,
        )?;
        Ok(Self { root })
    }

    fn root(&self) -> &Path {
        self.root.path()
    }
}

fn write(root: &Path, rel_path: &str, contents: &str) -> Result<()> {
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
