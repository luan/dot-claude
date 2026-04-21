use anyhow::{Result, anyhow, bail};
use tree_sitter::{Language, Node, Parser};

use crate::symbols::{
    Import, ParseResult, REF_KIND_CALL, REF_KIND_IMPLEMENTS, REF_KIND_USE, Ref, Symbol,
};

pub fn parse_source(source: &[u8], file_path: &str, language: &str) -> Result<ParseResult> {
    let ts_language = tree_sitter_language(language)?;
    let mut parser = Parser::new();
    parser
        .set_language(&ts_language)
        .map_err(|error| anyhow!("setting parser language for {language}: {error}"))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow!("parsing source for {file_path}"))?;

    let mut extractor = Extractor {
        source,
        file_path,
        language,
        result: ParseResult::default(),
    };
    extractor.walk(tree.root_node(), "", 0);
    Ok(extractor.result)
}

fn tree_sitter_language(language: &str) -> Result<Language> {
    match language {
        "bash" => Ok(tree_sitter_bash::LANGUAGE.into()),
        "c" => Ok(tree_sitter_c::LANGUAGE.into()),
        "cpp" => Ok(tree_sitter_cpp::LANGUAGE.into()),
        "go" => Ok(tree_sitter_go::LANGUAGE.into()),
        "python" => Ok(tree_sitter_python::LANGUAGE.into()),
        "javascript" => Ok(tree_sitter_javascript::LANGUAGE.into()),
        "typescript" => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        "tsx" => Ok(tree_sitter_typescript::LANGUAGE_TSX.into()),
        "java" => Ok(tree_sitter_java::LANGUAGE.into()),
        "kotlin" => Ok(tree_sitter_kotlin_ng::LANGUAGE.into()),
        "lua" => Ok(tree_sitter_lua::LANGUAGE.into()),
        "php" => Ok(tree_sitter_php::LANGUAGE_PHP.into()),
        "ruby" => Ok(tree_sitter_ruby::LANGUAGE.into()),
        "scala" => Ok(tree_sitter_scala::LANGUAGE.into()),
        "csharp" => Ok(tree_sitter_c_sharp::LANGUAGE.into()),
        "rust" => Ok(tree_sitter_rust::LANGUAGE.into()),
        "swift" => Ok(tree_sitter_swift::LANGUAGE.into()),
        _ => bail!("unsupported language: {language}"),
    }
}

struct Extractor<'a> {
    source: &'a [u8],
    file_path: &'a str,
    language: &'a str,
    result: ParseResult,
}

impl<'a> Extractor<'a> {
    fn walk(&mut self, node: Node<'_>, parent: &str, depth: usize) {
        if let Some(import) = self.extract_import(node) {
            self.result.imports.push(import);
        }
        for reference in self.extract_refs(node) {
            self.result.refs.push(reference);
        }

        let mut next_parent = parent.to_string();
        let mut next_depth = depth;
        if let Some(symbol) = self.node_to_symbol(node, parent, depth) {
            next_parent = symbol.name.clone();
            next_depth = depth + 1;
            self.result.symbols.push(symbol);
        }

        let count = node.child_count();
        for index in 0..count {
            if let Some(child) = node.child(index) {
                self.walk(child, &next_parent, next_depth);
            }
        }
    }

    fn node_to_symbol(&self, node: Node<'_>, parent: &str, depth: usize) -> Option<Symbol> {
        let (kind, name_node) = self.classify_node(node)?;
        let name = node_text(self.source, name_node)?;
        let signature = self.extract_signature(node, &kind);
        let start = node.start_position();
        let end = node.end_position();

        Some(Symbol {
            name,
            kind,
            file: self.file_path.to_string(),
            start_line: start.row + 1,
            end_line: end.row + 1,
            start_col: start.column,
            end_col: end.column,
            parent: parent.to_string(),
            depth,
            signature,
            language: self.language.to_string(),
        })
    }

    fn classify_node<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match self.language {
            "bash" => self.classify_bash(node),
            "c" => self.classify_c(node),
            "cpp" => self.classify_cpp(node),
            "go" => self.classify_go(node),
            "python" => self.classify_python(node),
            "javascript" | "typescript" | "tsx" => self.classify_js(node),
            "java" => self.classify_java(node),
            "kotlin" => self.classify_kotlin(node),
            "lua" => self.classify_lua(node),
            "php" => self.classify_php(node),
            "ruby" => self.classify_ruby(node),
            "scala" => self.classify_scala(node),
            "csharp" => self.classify_csharp(node),
            "rust" => self.classify_rust(node),
            "swift" => self.classify_swift(node),
            _ => None,
        }
    }

    fn classify_go<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_declaration" => {
                Some(("function".to_string(), node.child_by_field_name("name")?))
            }
            "method_declaration" => Some(("method".to_string(), node.child_by_field_name("name")?)),
            "type_spec" => {
                let type_node = node.child_by_field_name("type")?;
                let kind = match type_node.kind() {
                    "struct_type" => "struct",
                    "interface_type" => "interface",
                    _ => "type",
                };
                Some((kind.to_string(), node.child_by_field_name("name")?))
            }
            "const_spec" => Some(("constant".to_string(), node.child_by_field_name("name")?)),
            "var_spec" => Some(("variable".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn classify_bash<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_definition" => Some(("function".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn classify_c<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        classify_c_family(node, false)
    }

    fn classify_cpp<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        classify_c_family(node, true)
    }

    fn classify_python<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_definition" => {
                if node
                    .parent()
                    .is_some_and(|parent| parent.kind() == "decorated_definition")
                {
                    return None;
                }
                let name_node = node.child_by_field_name("name")?;
                let name = node_text(self.source, name_node)?;
                if name.starts_with('_') && name != "__init__" {
                    return None;
                }
                Some(("function".to_string(), name_node))
            }
            "class_definition" => {
                if node
                    .parent()
                    .is_some_and(|parent| parent.kind() == "decorated_definition")
                {
                    return None;
                }
                Some(("class".to_string(), node.child_by_field_name("name")?))
            }
            "decorated_definition" => {
                let count = node.child_count();
                for index in 0..count {
                    if let Some(child) = node.child(index) {
                        if let Some(classified) = self.classify_python_inner(child) {
                            return Some(classified);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn classify_python_inner<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_definition" => {
                let name_node = node.child_by_field_name("name")?;
                let name = node_text(self.source, name_node)?;
                if name.starts_with('_') && name != "__init__" {
                    return None;
                }
                Some(("function".to_string(), name_node))
            }
            "class_definition" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn classify_js<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_declaration"
            | "class_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "enum_declaration"
            | "lexical_declaration" => {
                if node
                    .parent()
                    .is_some_and(|parent| parent.kind() == "export_statement")
                {
                    return None;
                }
                self.classify_js_inner(node)
            }
            "method_definition" => Some(("method".to_string(), node.child_by_field_name("name")?)),
            "export_statement" => {
                let count = node.child_count();
                for index in 0..count {
                    if let Some(child) = node.child(index) {
                        if let Some(classified) = self.classify_js_inner(child) {
                            return Some(classified);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn classify_js_inner<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_declaration" => {
                Some(("function".to_string(), node.child_by_field_name("name")?))
            }
            "class_declaration" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            "interface_declaration" => {
                Some(("interface".to_string(), node.child_by_field_name("name")?))
            }
            "type_alias_declaration" => {
                Some(("type".to_string(), node.child_by_field_name("name")?))
            }
            "enum_declaration" => Some(("enum".to_string(), node.child_by_field_name("name")?)),
            "lexical_declaration" => {
                let count = node.child_count();
                for index in 0..count {
                    let child = node.child(index)?;
                    if child.kind() != "variable_declarator" {
                        continue;
                    }
                    let value_node = child.child_by_field_name("value")?;
                    if matches!(value_node.kind(), "arrow_function" | "function") {
                        return Some(("function".to_string(), child.child_by_field_name("name")?));
                    }
                }
                None
            }
            _ => None,
        }
    }

    fn classify_rust<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_item" => Some(("function".to_string(), node.child_by_field_name("name")?)),
            "struct_item" => Some(("struct".to_string(), node.child_by_field_name("name")?)),
            "enum_item" => Some(("enum".to_string(), node.child_by_field_name("name")?)),
            "trait_item" => Some(("trait".to_string(), node.child_by_field_name("name")?)),
            "type_item" => Some(("type".to_string(), node.child_by_field_name("name")?)),
            "const_item" => Some(("constant".to_string(), node.child_by_field_name("name")?)),
            "static_item" => Some(("variable".to_string(), node.child_by_field_name("name")?)),
            "mod_item" => Some(("module".to_string(), node.child_by_field_name("name")?)),
            "impl_item" => {
                let mut type_node = node.child_by_field_name("type")?;
                if type_node.kind() == "generic_type" {
                    if let Some(inner) = type_node.child_by_field_name("type") {
                        type_node = inner;
                    }
                }
                Some(("impl".to_string(), type_node))
            }
            _ => None,
        }
    }

    fn classify_java<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class_declaration" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            "interface_declaration" => {
                Some(("interface".to_string(), node.child_by_field_name("name")?))
            }
            "method_declaration" => Some(("method".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn classify_lua<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "function_declaration" => {
                Some(("function".to_string(), node.child_by_field_name("name")?))
            }
            _ => None,
        }
    }

    fn classify_php<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class_declaration" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            "interface_declaration" => {
                Some(("interface".to_string(), node.child_by_field_name("name")?))
            }
            "trait_declaration" => Some(("trait".to_string(), node.child_by_field_name("name")?)),
            "enum_declaration" => Some(("enum".to_string(), node.child_by_field_name("name")?)),
            "function_definition" => {
                Some(("function".to_string(), node.child_by_field_name("name")?))
            }
            "method_declaration" => Some(("method".to_string(), node.child_by_field_name("name")?)),
            "namespace_definition" => {
                Some(("module".to_string(), node.child_by_field_name("name")?))
            }
            _ => None,
        }
    }

    fn classify_ruby<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            "module" => Some(("module".to_string(), node.child_by_field_name("name")?)),
            "method" | "singleton_method" => {
                Some(("method".to_string(), node.child_by_field_name("name")?))
            }
            _ => None,
        }
    }

    fn classify_scala<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class_definition" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            "trait_definition" => Some(("trait".to_string(), node.child_by_field_name("name")?)),
            "object_definition" => Some(("object".to_string(), node.child_by_field_name("name")?)),
            "enum_definition" => Some(("enum".to_string(), node.child_by_field_name("name")?)),
            "function_definition" | "function_declaration" => {
                Some(("function".to_string(), node.child_by_field_name("name")?))
            }
            "type_definition" => Some(("type".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn classify_kotlin<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class_declaration" => Some((
                "class".to_string(),
                node.child_by_field_name("name")?,
            )),
            "function_declaration" => {
                Some(("function".to_string(), node.child_by_field_name("name")?))
            }
            _ => None,
        }
    }

    fn classify_csharp<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class_declaration" => Some(("class".to_string(), node.child_by_field_name("name")?)),
            "interface_declaration" => {
                Some(("interface".to_string(), node.child_by_field_name("name")?))
            }
            "struct_declaration" => {
                Some(("struct".to_string(), node.child_by_field_name("name")?))
            }
            "record_declaration" => {
                Some(("record".to_string(), node.child_by_field_name("name")?))
            }
            "method_declaration" => Some(("method".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn classify_swift<'tree>(&self, node: Node<'tree>) -> Option<(String, Node<'tree>)> {
        match node.kind() {
            "class_declaration" => {
                let kind = match node.child_by_field_name("declaration_kind")?.kind() {
                    "struct" => "struct",
                    "enum" => "enum",
                    "extension" => "extension",
                    "actor" => "actor",
                    _ => "class",
                };
                Some((kind.to_string(), node.child_by_field_name("name")?))
            }
            "protocol_declaration" => {
                Some(("protocol".to_string(), node.child_by_field_name("name")?))
            }
            "function_declaration" => Some(("function".to_string(), node.child_by_field_name("name")?)),
            _ => None,
        }
    }

    fn extract_import(&self, node: Node<'_>) -> Option<Import> {
        match self.language {
            "c" | "cpp" if node.kind() == "preproc_include" => {
                let raw = node
                    .child_by_field_name("path")
                    .and_then(|path| trimmed_node_text(self.source, path, "\"<>") )
                    .or_else(|| node_text(self.source, node))?;
                Some(Import {
                    raw_path: raw,
                    language: self.language.to_string(),
                })
            }
            "go" if node.kind() == "import_spec" => {
                let raw = trimmed_node_text(self.source, node.child_by_field_name("path")?, "\"")?;
                Some(Import {
                    raw_path: raw,
                    language: self.language.to_string(),
                })
            }
            "python" if matches!(node.kind(), "import_statement" | "import_from_statement") => {
                Some(Import {
                    raw_path: node_text(self.source, node)?,
                    language: self.language.to_string(),
                })
            }
            "javascript" | "typescript" | "tsx" if node.kind() == "import_statement" => {
                let raw =
                    trimmed_node_text(self.source, node.child_by_field_name("source")?, "\"'`")?;
                Some(Import {
                    raw_path: raw,
                    language: self.language.to_string(),
                })
            }
            "rust" if node.kind() == "use_declaration" => Some(Import {
                raw_path: node_text(self.source, node)?,
                language: self.language.to_string(),
            }),
            "java" if node.kind() == "import_declaration" => Some(Import {
                raw_path: node_text(self.source, node)?,
                language: self.language.to_string(),
            }),
            "csharp" if node.kind() == "using_directive" => Some(Import {
                raw_path: node_text(self.source, node)?,
                language: self.language.to_string(),
            }),
            "swift" if node.kind() == "import_declaration" => Some(Import {
                raw_path: node_text(self.source, node)?,
                language: self.language.to_string(),
            }),
            "php" if node.kind() == "namespace_use_declaration" => Some(Import {
                raw_path: node_text(self.source, node)?,
                language: self.language.to_string(),
            }),
            "ruby" if node.kind() == "call" => extract_ruby_import(self.source, node).map(|raw_path| Import {
                raw_path,
                language: self.language.to_string(),
            }),
            "bash" if node.kind() == "command" => extract_bash_import(self.source, node).map(|raw_path| Import {
                raw_path,
                language: self.language.to_string(),
            }),
            "lua" if node.kind() == "function_call" => extract_lua_import(self.source, node).map(|raw_path| Import {
                raw_path,
                language: self.language.to_string(),
            }),
            "scala" if node.kind() == "import_declaration" => Some(Import {
                raw_path: node_text(self.source, node)?,
                language: self.language.to_string(),
            }),
            _ => None,
        }
    }

    fn extract_refs(&self, node: Node<'_>) -> Vec<Ref> {
        match self.language {
            "bash" => self.extract_bash_refs(node),
            "c" => self.extract_c_refs(node),
            "cpp" => self.extract_cpp_refs(node),
            "go" => self.extract_go_refs(node),
            "python" => self.extract_python_refs(node),
            "javascript" | "typescript" | "tsx" => self.extract_js_refs(node),
            "java" => self.extract_java_refs(node),
            "kotlin" => self.extract_kotlin_refs(node),
            "lua" => self.extract_lua_refs(node),
            "php" => self.extract_php_refs(node),
            "ruby" => self.extract_ruby_refs(node),
            "scala" => self.extract_scala_refs(node),
            "csharp" => self.extract_csharp_refs(node),
            "rust" => self.extract_rust_refs(node),
            "swift" => self.extract_swift_refs(node),
            _ => Vec::new(),
        }
    }

    fn extract_bash_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let Some(reference) = extract_bash_command_ref(self.source, node, self.language) else {
            return Vec::new();
        };
        vec![reference]
    }

    fn extract_c_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "call_expression", "function") {
            refs.push(reference);
        }
        refs
    }

    fn extract_cpp_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = self.extract_c_refs(node);
        if matches!(node.kind(), "class_specifier" | "struct_specifier") {
            refs.extend(self.extract_child_clause(node, "base_class_clause"));
        }
        refs
    }

    fn extract_go_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "call_expression", "function") {
            refs.push(reference);
        }
        if node.kind() == "composite_literal" {
            if let Some(reference) = self.extract_go_use_ref(node) {
                refs.push(reference);
            }
        }
        if node.kind() == "type_spec" {
            refs.extend(self.extract_go_interface_embeds(node));
        }
        refs
    }

    fn extract_go_use_ref(&self, node: Node<'_>) -> Option<Ref> {
        let type_node = node.child_by_field_name("type")?;
        let line = node.start_position().row + 1;
        let name = match type_node.kind() {
            "type_identifier" => node_text(self.source, type_node)?,
            "qualified_type" => node_text(self.source, type_node.child_by_field_name("name")?)?,
            _ => return None,
        };

        Some(Ref {
            name,
            line,
            language: self.language.to_string(),
            kind: REF_KIND_USE.to_string(),
        })
    }

    fn extract_go_interface_embeds(&self, node: Node<'_>) -> Vec<Ref> {
        let Some(type_node) = node.child_by_field_name("type") else {
            return Vec::new();
        };
        if type_node.kind() != "interface_type" {
            return Vec::new();
        }

        let mut refs = Vec::new();
        collect_go_embedded_types(self.source, type_node, &mut refs, self.language);
        refs
    }

    fn extract_python_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "call", "function") {
            refs.push(reference);
        }
        if node.kind() == "class_definition" {
            refs.extend(self.extract_python_implements(node));
        }
        refs
    }

    fn extract_python_implements(&self, node: Node<'_>) -> Vec<Ref> {
        let superclasses = node
            .child_by_field_name("superclasses")
            .or_else(|| find_child_by_kind(node, "argument_list"));
        let Some(superclasses) = superclasses else {
            return Vec::new();
        };

        let mut refs = Vec::new();
        let count = superclasses.named_child_count();
        for index in 0..count {
            if let Some(child) = superclasses.named_child(index) {
                if let Some(reference) = self.implements_ref(child, node.start_position().row + 1) {
                    refs.push(reference);
                }
            }
        }
        refs
    }

    fn extract_js_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "call_expression", "function") {
            refs.push(reference);
        }
        if node.kind() == "new_expression" {
            if let Some(reference) = self.extract_js_new_ref(node) {
                refs.push(reference);
            }
        }
        if matches!(node.kind(), "class_declaration" | "class" | "interface_declaration") {
            refs.extend(self.extract_js_implements(node));
        }
        refs
    }

    fn extract_js_new_ref(&self, node: Node<'_>) -> Option<Ref> {
        let constructor = node.child_by_field_name("constructor")?;
        let name = extract_call_name(self.source, constructor)?;
        Some(Ref {
            name,
            line: node.start_position().row + 1,
            language: self.language.to_string(),
            kind: REF_KIND_CALL.to_string(),
        })
    }

    fn extract_js_implements(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        collect_clause_refs(
            self.source,
            node,
            &["class_heritage", "extends_clause", "implements_clause", "extends_type_clause"],
            node.start_position().row + 1,
            self.language,
            &mut refs,
        );
        refs
    }

    fn extract_rust_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "call_expression", "function") {
            refs.push(reference);
        }
        if node.kind() == "impl_item" {
            if let Some(trait_node) = node.child_by_field_name("trait") {
                if let Some(reference) = self.implements_ref(trait_node, node.start_position().row + 1) {
                    refs.push(reference);
                }
            }
        }
        refs
    }

    fn extract_java_refs(&self, node: Node<'_>) -> Vec<Ref> {
        if matches!(node.kind(), "class_declaration" | "interface_declaration") {
            return self.extract_named_clauses(
                node,
                &["superclass", "interfaces", "extends_interfaces"],
            );
        }
        Vec::new()
    }

    fn extract_kotlin_refs(&self, node: Node<'_>) -> Vec<Ref> {
        if node.kind() != "class_declaration" {
            return Vec::new();
        }

        let mut refs = Vec::new();
        let count = node.named_child_count();
        for index in 0..count {
            if let Some(child) = node.named_child(index) {
                if matches!(child.kind(), "delegation_specifier" | "delegation_specifiers") {
                    collect_clause_items(
                        self.source,
                        child,
                        node.start_position().row + 1,
                        self.language,
                        &mut refs,
                    );
                }
            }
        }
        refs
    }

    fn extract_csharp_refs(&self, node: Node<'_>) -> Vec<Ref> {
        if matches!(
            node.kind(),
            "class_declaration" | "interface_declaration" | "struct_declaration" | "record_declaration"
        ) {
            return self.extract_child_clause(node, "base_list");
        }
        Vec::new()
    }

    fn extract_swift_refs(&self, node: Node<'_>) -> Vec<Ref> {
        if matches!(node.kind(), "class_declaration" | "protocol_declaration") {
            return self.extract_child_clause(node, "inheritance_specifier");
        }
        Vec::new()
    }

    fn extract_lua_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "function_call", "name") {
            if reference.name != "require" {
                refs.push(reference);
            }
        }
        refs
    }

    fn extract_php_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if matches!(node.kind(), "class_declaration" | "interface_declaration") {
            refs.extend(self.extract_child_clause(node, "base_clause"));
            refs.extend(self.extract_child_clause(node, "class_interface_clause"));
        }
        if let Some(reference) = self.extract_php_call_ref(node) {
            refs.push(reference);
        }
        refs
    }

    fn extract_php_call_ref(&self, node: Node<'_>) -> Option<Ref> {
        let target = match node.kind() {
            "function_call_expression" => node.child_by_field_name("function")?,
            "member_call_expression" | "scoped_call_expression" => {
                node.child_by_field_name("name")
                    .or_else(|| node.child_by_field_name("member_name"))?
            }
            "object_creation_expression" => node.child_by_field_name("class")?,
            _ => return None,
        };
        let name = extract_call_name(self.source, target)?;
        Some(Ref {
            name,
            line: node.start_position().row + 1,
            language: self.language.to_string(),
            kind: REF_KIND_CALL.to_string(),
        })
    }

    fn extract_ruby_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if node.kind() == "class" {
            if let Some(superclass) = node.child_by_field_name("superclass") {
                if let Some(reference) = self.implements_ref(superclass, node.start_position().row + 1) {
                    refs.push(reference);
                }
            }
        }
        if let Some(reference) = extract_ruby_call_ref(self.source, node, self.language) {
            refs.push(reference);
        }
        refs
    }

    fn extract_scala_refs(&self, node: Node<'_>) -> Vec<Ref> {
        let mut refs = Vec::new();
        if let Some(reference) = self.extract_call_ref(node, "call_expression", "function") {
            refs.push(reference);
        }
        if node.kind() == "instance_expression" {
            if let Some(reference) = self.extract_call_ref(node, "instance_expression", "function") {
                refs.push(reference);
            }
        }
        if matches!(node.kind(), "class_definition" | "trait_definition" | "object_definition") {
            refs.extend(self.extract_child_clause(node, "extends_clause"));
        }
        refs
    }

    fn extract_named_clauses(&self, node: Node<'_>, fields: &[&str]) -> Vec<Ref> {
        let mut refs = Vec::new();
        for field in fields {
            if let Some(child) = node.child_by_field_name(field) {
                collect_clause_items(
                    self.source,
                    child,
                    node.start_position().row + 1,
                    self.language,
                    &mut refs,
                );
            }
        }
        refs
    }

    fn extract_child_clause(&self, node: Node<'_>, child_kind: &str) -> Vec<Ref> {
        let mut refs = Vec::new();
        let count = node.named_child_count();
        for index in 0..count {
            if let Some(child) = node.named_child(index) {
                if child.kind() == child_kind {
                    collect_clause_items(
                        self.source,
                        child,
                        node.start_position().row + 1,
                        self.language,
                        &mut refs,
                    );
                }
            }
        }
        refs
    }

    fn extract_call_ref(&self, node: Node<'_>, call_kind: &str, field: &str) -> Option<Ref> {
        if node.kind() != call_kind {
            return None;
        }
        let function = node.child_by_field_name(field)?;
        let name = extract_call_name(self.source, function)?;
        Some(Ref {
            name,
            line: node.start_position().row + 1,
            language: self.language.to_string(),
            kind: REF_KIND_CALL.to_string(),
        })
    }

    fn extract_signature(&self, node: Node<'_>, kind: &str) -> String {
        match kind {
            "function" | "method" => {
                let mut signature = node
                    .child_by_field_name("parameters")
                    .and_then(|parameters| node_text(self.source, parameters))
                    .unwrap_or_default();
                if let Some(return_type) = node.child_by_field_name("return_type") {
                    if let Some(return_type) = node_text(self.source, return_type) {
                        match self.language {
                            "python" => signature.push_str(&format!(" -> {return_type}")),
                            "go" => signature.push_str(&format!(" {return_type}")),
                            _ => signature.push_str(&return_type),
                        }
                    }
                }
                signature
            }
            "struct" | "class" | "interface" | "trait" | "enum" => node_text(self.source, node)
                .map(|text| {
                    text.split(['\n', '{'])
                        .next()
                        .unwrap_or_default()
                        .trim()
                        .to_string()
                })
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    fn implements_ref(&self, node: Node<'_>, line: usize) -> Option<Ref> {
        Some(Ref {
            name: extract_call_name(self.source, node)?,
            line,
            language: self.language.to_string(),
            kind: REF_KIND_IMPLEMENTS.to_string(),
        })
    }
}

fn node_text(source: &[u8], node: Node<'_>) -> Option<String> {
    node.utf8_text(source).ok().map(ToOwned::to_owned)
}

fn trimmed_node_text(source: &[u8], node: Node<'_>, trim_chars: &str) -> Option<String> {
    node_text(source, node).map(|text| text.trim_matches(|c| trim_chars.contains(c)).to_string())
}

fn extract_call_name(source: &[u8], node: Node<'_>) -> Option<String> {
    match node.kind() {
        "identifier"
        | "type_identifier"
        | "property_identifier"
        | "field_identifier"
        | "name"
        | "relative_name"
        | "constant" => {
            node_text(source, node)
        }
        "selector_expression"
        | "member_expression"
        | "field_expression"
        | "scoped_identifier"
        | "scope_resolution"
        | "qualified_name"
        | "attribute"
        | "generic_type" => {
            node.child_by_field_name("field")
                .or_else(|| node.child_by_field_name("attribute"))
                .or_else(|| node.child_by_field_name("property"))
                .or_else(|| node.child_by_field_name("name"))
                .or_else(|| node.child_by_field_name("type"))
                .and_then(|child| node_text(source, child))
        }
        _ => {
            let count = node.named_child_count();
            for index in 0..count {
                if let Some(child) = node.named_child(index) {
                    if let Some(name) = extract_call_name(source, child) {
                        return Some(name);
                    }
                }
            }
            None
        }
    }
}

fn find_child_by_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let count = node.named_child_count();
    for index in 0..count {
        let child = node.named_child(index)?;
        if child.kind() == kind {
            return Some(child);
        }
    }
    None
}

fn collect_go_embedded_types(source: &[u8], node: Node<'_>, refs: &mut Vec<Ref>, language: &str) {
    if node.kind() == "type_elem" {
        let count = node.named_child_count();
        for index in 0..count {
            if let Some(child) = node.named_child(index) {
                if let Some(name) = extract_call_name(source, child) {
                    refs.push(Ref {
                        name,
                        line: child.start_position().row + 1,
                        language: language.to_string(),
                        kind: REF_KIND_IMPLEMENTS.to_string(),
                    });
                    return;
                }
            }
        }
    }

    let count = node.named_child_count();
    for index in 0..count {
        if let Some(child) = node.named_child(index) {
            collect_go_embedded_types(source, child, refs, language);
        }
    }
}

fn collect_clause_refs(
    source: &[u8],
    node: Node<'_>,
    clause_kinds: &[&str],
    line: usize,
    language: &str,
    refs: &mut Vec<Ref>,
) {
    let count = node.named_child_count();
    for index in 0..count {
        if let Some(child) = node.named_child(index) {
            if clause_kinds.contains(&child.kind()) {
                collect_clause_items(source, child, line, language, refs);
                continue;
            }
            collect_clause_refs(source, child, clause_kinds, line, language, refs);
        }
    }
}

fn collect_clause_items(
    source: &[u8],
    node: Node<'_>,
    line: usize,
    language: &str,
    refs: &mut Vec<Ref>,
) {
    if is_type_name_node(node.kind()) {
        if let Some(name) = extract_call_name(source, node) {
            refs.push(Ref {
                name,
                line,
                language: language.to_string(),
                kind: REF_KIND_IMPLEMENTS.to_string(),
            });
        }
        return;
    }

    let count = node.named_child_count();
    for index in 0..count {
        if let Some(child) = node.named_child(index) {
            collect_clause_items(source, child, line, language, refs);
        }
    }
}

fn is_type_name_node(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "type_identifier"
            | "property_identifier"
            | "field_identifier"
            | "name"
            | "relative_name"
            | "constant"
            | "selector_expression"
            | "member_expression"
            | "field_expression"
            | "scope_resolution"
            | "scoped_identifier"
            | "qualified_name"
            | "attribute"
            | "generic_type"
    )
}

fn classify_c_family<'tree>(node: Node<'tree>, cpp: bool) -> Option<(String, Node<'tree>)> {
    match node.kind() {
        "function_definition" => Some((
            "function".to_string(),
            extract_c_family_name_node(node.child_by_field_name("declarator")?)?,
        )),
        "struct_specifier" => Some(("struct".to_string(), node.child_by_field_name("name")?)),
        "union_specifier" => Some(("type".to_string(), node.child_by_field_name("name")?)),
        "enum_specifier" => Some(("enum".to_string(), node.child_by_field_name("name")?)),
        "type_definition" => Some((
            "type".to_string(),
            extract_c_family_name_node(node.child_by_field_name("declarator")?)?,
        )),
        "class_specifier" if cpp => Some(("class".to_string(), node.child_by_field_name("name")?)),
        "namespace_definition" if cpp => {
            Some(("module".to_string(), node.child_by_field_name("name")?))
        }
        _ => None,
    }
}

fn extract_c_family_name_node<'tree>(node: Node<'tree>) -> Option<Node<'tree>> {
    match node.kind() {
        "identifier" | "field_identifier" | "type_identifier" => Some(node),
        _ => {
            let count = node.named_child_count();
            for index in 0..count {
                let child = node.named_child(index)?;
                if let Some(name_node) = extract_c_family_name_node(child) {
                    return Some(name_node);
                }
            }
            None
        }
    }
}

fn extract_ruby_import(source: &[u8], node: Node<'_>) -> Option<String> {
    if node.kind() != "call" {
        return None;
    }
    let method = node.child_by_field_name("method")?;
    let method_name = node_text(source, method)?;
    if !matches!(method_name.as_str(), "require" | "require_relative") {
        return None;
    }
    let arguments = node.child_by_field_name("arguments")?;
    let first = arguments.named_child(0)?;
    trimmed_node_text(source, first, "\"'")
}

fn extract_ruby_call_ref(source: &[u8], node: Node<'_>, language: &str) -> Option<Ref> {
    if node.kind() != "call" {
        return None;
    }
    let method = node.child_by_field_name("method")?;
    let name = node_text(source, method)?;
    if matches!(name.as_str(), "require" | "require_relative") {
        return None;
    }
    Some(Ref {
        name,
        line: node.start_position().row + 1,
        language: language.to_string(),
        kind: REF_KIND_CALL.to_string(),
    })
}

fn extract_bash_import(source: &[u8], node: Node<'_>) -> Option<String> {
    if node.kind() != "command" {
        return None;
    }
    let name = node.named_child(0)?;
    let command = node_text(source, name)?;
    if !matches!(command.as_str(), "source" | ".") {
        return None;
    }
    let arg = node.named_child(1)?;
    trimmed_node_text(source, arg, "\"'")
}

fn extract_bash_command_ref(source: &[u8], node: Node<'_>, language: &str) -> Option<Ref> {
    if node.kind() != "command" {
        return None;
    }
    let name = node.named_child(0)?;
    let command = node_text(source, name)?;
    if matches!(command.as_str(), "source" | ".") {
        return None;
    }
    Some(Ref {
        name: command,
        line: node.start_position().row + 1,
        language: language.to_string(),
        kind: REF_KIND_CALL.to_string(),
    })
}

fn extract_lua_import(source: &[u8], node: Node<'_>) -> Option<String> {
    if node.kind() != "function_call" {
        return None;
    }
    let name = node.child_by_field_name("name")?;
    if node_text(source, name)? != "require" {
        return None;
    }
    let arguments = node.child_by_field_name("arguments")?;
    let first = arguments.named_child(0)?;
    trimmed_node_text(source, first, "\"'")
}
