#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
    pub parent: String,
    pub depth: usize,
    pub signature: String,
    pub language: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub raw_path: String,
    pub language: String,
}

pub const REF_KIND_CALL: &str = "call";
pub const REF_KIND_IMPLEMENTS: &str = "implements";
pub const REF_KIND_USE: &str = "use";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub name: String,
    pub line: usize,
    pub language: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseResult {
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub refs: Vec<Ref>,
}
