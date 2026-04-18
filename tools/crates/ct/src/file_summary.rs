use std::fs;
use std::path::{Path, PathBuf};

use tree_sitter::{Node, Parser};

#[derive(Clone, Copy)]
enum Lang {
    Rust,
    Swift,
}

struct Symbol {
    label: &'static str,
    name: String,
    signature: Option<String>,
    fields: Vec<String>,
    doc: Option<String>,
    line: usize,
}

const MAX_DEPTH: u32 = 2;
const MAX_SIG_CHARS: usize = 160;
const MAX_FIELDS: usize = 8;
const MAX_DOC_CHARS: usize = 200;

impl Lang {
    fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|e| e.to_str())? {
            "rs" => Some(Lang::Rust),
            "swift" => Some(Lang::Swift),
            _ => None,
        }
    }

    fn ts_language(self) -> tree_sitter::Language {
        match self {
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::Swift => tree_sitter_swift::LANGUAGE.into(),
        }
    }

    fn label_for(self, kind: &str) -> Option<&'static str> {
        Some(match (self, kind) {
            (Lang::Rust, "function_item") => "FN",
            (Lang::Rust, "struct_item") => "STRUCT",
            (Lang::Rust, "enum_item") => "ENUM",
            (Lang::Rust, "trait_item") => "TRAIT",
            (Lang::Rust, "impl_item") => "IMPL",
            (Lang::Rust, "mod_item") => "MOD",
            (Lang::Swift, "function_declaration") => "FN",
            (Lang::Swift, "class_declaration") => "CLASS",
            (Lang::Swift, "protocol_declaration") => "PROTOCOL",
            (Lang::Swift, "typealias_declaration") => "TYPEALIAS",
            _ => return None,
        })
    }

    fn is_function(self, kind: &str) -> bool {
        matches!(
            (self, kind),
            (Lang::Rust, "function_item") | (Lang::Swift, "function_declaration")
        )
    }

    fn is_struct_like(self, kind: &str) -> bool {
        matches!(
            (self, kind),
            (Lang::Rust, "struct_item") | (Lang::Swift, "class_declaration")
        )
    }
}

pub fn run(path_str: String) -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(&path_str);
    let Some(lang) = Lang::from_path(&path) else {
        return Err(format!("unsupported file extension: {}", path.display()).into());
    };

    let source = fs::read_to_string(&path)?;
    let symbols = extract_symbols(&source, lang)?;

    println!("{} ({} lines)", path.display(), source.lines().count());
    for s in &symbols {
        if let Some(doc) = &s.doc {
            println!("    /// {doc}");
        }
        match &s.signature {
            Some(sig) => println!("{} {}{} L{}", s.label, s.name, sig, s.line),
            None => println!("{} {} L{}", s.label, s.name, s.line),
        }
        for field in &s.fields {
            println!("    {field}");
        }
    }
    Ok(())
}

fn extract_symbols(source: &str, lang: Lang) -> Result<Vec<Symbol>, Box<dyn std::error::Error>> {
    let mut parser = Parser::new();
    parser.set_language(&lang.ts_language())?;
    let tree = parser
        .parse(source, None)
        .ok_or("tree-sitter returned no tree")?;

    let mut out = Vec::new();
    walk(tree.root_node(), source.as_bytes(), lang, &mut out, 0);
    Ok(out)
}

fn walk(node: Node, source: &[u8], lang: Lang, out: &mut Vec<Symbol>, depth: u32) {
    if depth > MAX_DEPTH {
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(label) = lang.label_for(child.kind()) {
            let kind = child.kind();
            let signature = lang
                .is_function(kind)
                .then(|| extract_signature(&child, source))
                .flatten();
            let fields = if lang.is_struct_like(kind) {
                extract_fields(&child, source)
            } else {
                Vec::new()
            };
            out.push(Symbol {
                label,
                name: node_name(&child, source),
                signature,
                fields,
                doc: extract_doc(&child, source),
                line: child.start_position().row + 1,
            });
            if let Some(body) = child.child_by_field_name("body") {
                walk(body, source, lang, out, depth + 1);
            }
        }
    }
}

fn node_name(node: &Node, source: &[u8]) -> String {
    let text = node
        .child_by_field_name("name")
        .or_else(|| node.child_by_field_name("type"))
        .and_then(|n| n.utf8_text(source).ok())
        .unwrap_or("?");
    flatten(text)
}

fn extract_signature(node: &Node, source: &[u8]) -> Option<String> {
    let params = node
        .child_by_field_name("parameters")
        .and_then(|n| n.utf8_text(source).ok())
        .map(flatten)?;
    let ret = node
        .child_by_field_name("return_type")
        .and_then(|n| n.utf8_text(source).ok())
        .map(flatten);
    let mut sig = params;
    if let Some(r) = ret {
        sig.push_str(" -> ");
        sig.push_str(&r);
    }
    Some(truncate(sig, MAX_SIG_CHARS))
}

fn extract_fields(node: &Node, source: &[u8]) -> Vec<String> {
    let Some(body) = node.child_by_field_name("body") else {
        return Vec::new();
    };
    let mut cursor = body.walk();
    let mut out = Vec::new();
    for child in body.children(&mut cursor) {
        if child.kind() == "field_declaration"
            && let Ok(text) = child.utf8_text(source)
        {
            out.push(flatten(text));
            if out.len() >= MAX_FIELDS {
                out.push("...".into());
                break;
            }
        }
    }
    out
}

fn extract_doc(node: &Node, source: &[u8]) -> Option<String> {
    let mut lines = Vec::new();
    let mut sib = node.prev_sibling();
    while let Some(n) = sib {
        if n.kind() != "line_comment" {
            break;
        }
        let Ok(text) = n.utf8_text(source) else {
            break;
        };
        let Some(stripped) = text
            .strip_prefix("///")
            .or_else(|| text.strip_prefix("//!"))
        else {
            break;
        };
        lines.push(stripped.trim().to_string());
        sib = n.prev_sibling();
    }
    if lines.is_empty() {
        return None;
    }
    lines.reverse();
    Some(truncate(lines.join(" "), MAX_DOC_CHARS))
}

fn flatten(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: String, max: usize) -> String {
    if s.chars().count() <= max {
        return s;
    }
    let mut out: String = s.chars().take(max).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find<'a>(syms: &'a [Symbol], name: &str) -> &'a Symbol {
        syms.iter().find(|s| s.name == name).expect("symbol")
    }

    #[test]
    fn rust_signature_and_return_type() {
        let src = "pub fn alpha(x: u32, y: &str) -> Result<u32, Error> { Ok(x) }";
        let syms = extract_symbols(src, Lang::Rust).unwrap();
        let sig = find(&syms, "alpha").signature.as_deref().unwrap();
        assert!(sig.contains("(x: u32, y: &str)"));
        assert!(sig.contains("-> Result<u32, Error>"));
    }

    #[test]
    fn rust_struct_fields() {
        let src = "pub struct Beta { pub y: String, count: usize }";
        let syms = extract_symbols(src, Lang::Rust).unwrap();
        let fields = &find(&syms, "Beta").fields;
        assert!(fields.iter().any(|f| f.contains("y: String")));
        assert!(fields.iter().any(|f| f.contains("count: usize")));
    }

    #[test]
    fn rust_doc_comment() {
        let src = "/// Increments x by one.\n\
/// Returns the new value.\n\
pub fn alpha(x: u32) -> u32 { x + 1 }";
        let syms = extract_symbols(src, Lang::Rust).unwrap();
        let doc = find(&syms, "alpha").doc.as_deref().unwrap();
        assert!(doc.contains("Increments x by one"));
        assert!(doc.contains("Returns the new value"));
    }

    #[test]
    fn rust_impl_method_walked_with_signature() {
        let src = "pub struct Beta;\n\
impl Beta { pub fn method(&self, n: u32) -> u32 { n } }";
        let syms = extract_symbols(src, Lang::Rust).unwrap();
        let method = find(&syms, "method");
        assert_eq!(method.label, "FN");
        assert!(method.signature.as_deref().unwrap().contains("-> u32"));
    }
}
