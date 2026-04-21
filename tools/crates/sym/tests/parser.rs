use anyhow::Result;

use sym::parser::parse_source;
use sym::symbols::{ParseResult, REF_KIND_IMPLEMENTS, REF_KIND_USE};

#[test]
fn parses_go_functions_methods_imports_refs_and_signature() -> Result<()> {
    let source = br#"package main

import "fmt"

type Server struct {
    Port int
}

func (s *Server) Start() error {
    fmt.Println("starting")
    return nil
}

func main() {
    s := Server{Port: 8080}
    s.Start()
}
"#;

    let result = parse_source(source, "test.go", "go")?;

    assert_has_symbol(&result, "Server", "struct");
    let start = assert_has_symbol(&result, "Start", "method");
    assert!(start.signature.contains("()"));
    assert!(
        result
            .imports
            .iter()
            .any(|import_| import_.raw_path == "fmt")
    );
    assert!(
        result
            .refs
            .iter()
            .any(|reference| reference.name == "Println")
    );
    assert!(
        result
            .refs
            .iter()
            .any(|reference| reference.name == "Start")
    );
    assert!(
        result
            .refs
            .iter()
            .any(|reference| reference.name == "Server" && reference.kind == REF_KIND_USE)
    );

    Ok(())
}

#[test]
fn parses_python_symbols_imports_and_private_skip() -> Result<()> {
    let source = br#"import os
from pathlib import Path

class Animal:
    def __init__(self, name):
        self.name = name

def public_func():
    return Animal("dog")

def _private_func():
    pass
"#;

    let result = parse_source(source, "test.py", "python")?;

    assert_has_symbol(&result, "Animal", "class");
    assert_has_symbol(&result, "__init__", "function");
    assert_has_symbol(&result, "public_func", "function");
    assert!(
        !result
            .symbols
            .iter()
            .any(|symbol| symbol.name == "_private_func")
    );
    assert!(
        result
            .imports
            .iter()
            .any(|import_| import_.raw_path.contains("import os"))
    );
    assert!(
        result
            .refs
            .iter()
            .any(|reference| reference.name == "Animal")
    );

    Ok(())
}

#[test]
fn parses_javascript_and_typescript_symbols_and_refs() -> Result<()> {
    let js = br#"export function greet(name) {
  return name
}

class UserService {
  getUser(id) {
    return id
  }
}

const helper = (x) => greet(x)

new UserService().getUser(1)
"#;
    let js_result = parse_source(js, "test.js", "javascript")?;
    assert_has_symbol(&js_result, "greet", "function");
    assert_has_symbol(&js_result, "UserService", "class");
    assert_has_symbol(&js_result, "getUser", "method");
    assert_has_symbol(&js_result, "helper", "function");
    assert!(
        js_result
            .refs
            .iter()
            .any(|reference| reference.name == "greet")
    );
    assert!(
        js_result
            .refs
            .iter()
            .any(|reference| reference.name == "UserService")
    );
    assert!(
        js_result
            .refs
            .iter()
            .any(|reference| reference.name == "getUser")
    );

    let ts = br#"interface Service {
  run(): void
}

type Id = string

enum Status {
  Ready,
}
"#;
    let ts_result = parse_source(ts, "test.ts", "typescript")?;
    assert_has_symbol(&ts_result, "Service", "interface");
    assert_has_symbol(&ts_result, "Id", "type");
    assert_has_symbol(&ts_result, "Status", "enum");

    let tsx = br#"import { Button } from "./button"

type Props = {
  label: string
}

export function App({ label }: Props) {
  return <Button label={label} />
}
"#;
    let tsx_result = parse_source(tsx, "App.tsx", "tsx")?;
    assert_has_symbol(&tsx_result, "Props", "type");
    assert_has_symbol(&tsx_result, "App", "function");
    assert!(
        tsx_result
            .imports
            .iter()
            .any(|import_| import_.raw_path == "./button")
    );

    Ok(())
}

#[test]
fn parses_rust_symbols() -> Result<()> {
    let source = br#"struct Config {
    value: i32,
}

trait Runner {}

fn execute() {}

impl Config {}
"#;
    let result = parse_source(source, "test.rs", "rust")?;

    assert_has_symbol(&result, "Config", "struct");
    assert_has_symbol(&result, "Runner", "trait");
    assert_has_symbol(&result, "execute", "function");
    assert_has_symbol(&result, "Config", "impl");

    Ok(())
}

#[test]
fn parses_implements_refs_across_supported_languages() -> Result<()> {
    let go = br#"package io

type ReadWriter interface {
    Reader
    Writer
}
"#;
    let go_result = parse_source(go, "io.go", "go")?;
    assert_has_ref(&go_result, "Reader", REF_KIND_IMPLEMENTS);
    assert_has_ref(&go_result, "Writer", REF_KIND_IMPLEMENTS);

    let ts = br#"class UserRepo extends BaseRepo implements IUserRepository, Serializable {}

interface Named extends Identifiable {}
"#;
    let ts_result = parse_source(ts, "repo.ts", "typescript")?;
    assert_has_ref(&ts_result, "BaseRepo", REF_KIND_IMPLEMENTS);
    assert_has_ref(&ts_result, "IUserRepository", REF_KIND_IMPLEMENTS);
    assert_has_ref(&ts_result, "Serializable", REF_KIND_IMPLEMENTS);
    assert_has_ref(&ts_result, "Identifiable", REF_KIND_IMPLEMENTS);

    let py = br#"import routing

class APIRouter(routing.Router, BaseRouter):
    pass
"#;
    let py_result = parse_source(py, "router.py", "python")?;
    assert_has_ref(&py_result, "Router", REF_KIND_IMPLEMENTS);
    assert_has_ref(&py_result, "BaseRouter", REF_KIND_IMPLEMENTS);
    assert!(
        !py_result
            .refs
            .iter()
            .any(|reference| reference.name == "routing" && reference.kind == REF_KIND_IMPLEMENTS)
    );

    let rust = br#"struct Cache;

impl Reader for Cache {}
"#;
    let rust_result = parse_source(rust, "cache.rs", "rust")?;
    assert_has_symbol(&rust_result, "Cache", "impl");
    assert_has_ref(&rust_result, "Reader", REF_KIND_IMPLEMENTS);

    Ok(())
}

#[test]
fn parses_java_kotlin_csharp_and_swift_core_symbols_and_conformance() -> Result<()> {
    let java = br#"package x;

public class MyRunner extends BaseRunner implements Runnable, AutoCloseable {
}
"#;
    let java_result = parse_source(java, "MyRunner.java", "java")?;
    assert_has_symbol(&java_result, "MyRunner", "class");
    assert_has_ref(&java_result, "BaseRunner", REF_KIND_IMPLEMENTS);
    assert_has_ref(&java_result, "Runnable", REF_KIND_IMPLEMENTS);
    assert_has_ref(&java_result, "AutoCloseable", REF_KIND_IMPLEMENTS);

    let kotlin = br#"class UserRepo : BaseRepo(), IUserRepository, AutoCloseable {
}
"#;
    let kotlin_result = parse_source(kotlin, "repo.kt", "kotlin")?;
    assert_has_symbol(&kotlin_result, "UserRepo", "class");
    assert_has_ref(&kotlin_result, "BaseRepo", REF_KIND_IMPLEMENTS);
    assert_has_ref(&kotlin_result, "IUserRepository", REF_KIND_IMPLEMENTS);
    assert_has_ref(&kotlin_result, "AutoCloseable", REF_KIND_IMPLEMENTS);

    let csharp = br#"namespace X;

public class UserRepo : BaseRepo, IUserRepository, IDisposable {
}
"#;
    let csharp_result = parse_source(csharp, "UserRepo.cs", "csharp")?;
    assert_has_symbol(&csharp_result, "UserRepo", "class");
    assert_has_ref(&csharp_result, "BaseRepo", REF_KIND_IMPLEMENTS);
    assert_has_ref(&csharp_result, "IUserRepository", REF_KIND_IMPLEMENTS);
    assert_has_ref(&csharp_result, "IDisposable", REF_KIND_IMPLEMENTS);

    let swift = br#"import Foundation

class TimerActivityIntent: LiveActivityIntent, Sendable {
}

protocol Named: Identifiable {
}
"#;
    let swift_result = parse_source(swift, "Timer.swift", "swift")?;
    assert_has_symbol(&swift_result, "TimerActivityIntent", "class");
    assert_has_symbol(&swift_result, "Named", "protocol");
    assert_has_ref(&swift_result, "LiveActivityIntent", REF_KIND_IMPLEMENTS);
    assert_has_ref(&swift_result, "Sendable", REF_KIND_IMPLEMENTS);
    assert_has_ref(&swift_result, "Identifiable", REF_KIND_IMPLEMENTS);

    Ok(())
}

#[test]
fn parses_remaining_parseable_languages_minimally() -> Result<()> {
    let c = br#"#include "worker.h"

typedef struct Worker {
    int id;
} Worker;

void run(void) {
    helper();
}
"#;
    let c_result = parse_source(c, "worker.c", "c")?;
    assert_has_symbol(&c_result, "run", "function");
    assert!(
        c_result
            .imports
            .iter()
            .any(|import_| import_.raw_path.contains("worker.h"))
    );
    assert!(c_result.refs.iter().any(|reference| reference.name == "helper"));

    let cpp = br#"#include "runner.hpp"

class Runner : public BaseRunner, public Runnable {
public:
    void start() {
        helper();
    }
};
"#;
    let cpp_result = parse_source(cpp, "runner.cpp", "cpp")?;
    assert_has_symbol(&cpp_result, "Runner", "class");
    assert_has_ref(&cpp_result, "BaseRunner", REF_KIND_IMPLEMENTS);
    assert_has_ref(&cpp_result, "Runnable", REF_KIND_IMPLEMENTS);
    assert!(cpp_result.refs.iter().any(|reference| reference.name == "helper"));

    let php = br#"<?php
namespace App;

use Lib\BaseRepo;

class UserRepo extends BaseRepo implements JsonSerializable {
    public function run(): void {
        helper();
    }
}
"#;
    let php_result = parse_source(php, "repo.php", "php")?;
    assert_has_symbol(&php_result, "UserRepo", "class");
    assert_has_symbol(&php_result, "run", "method");
    assert_has_ref(&php_result, "BaseRepo", REF_KIND_IMPLEMENTS);
    assert_has_ref(&php_result, "JsonSerializable", REF_KIND_IMPLEMENTS);
    assert!(php_result.refs.iter().any(|reference| reference.name == "helper"));

    let ruby = br#"require "json"

class Worker < BaseWorker
  def run
    helper()
  end
end
"#;
    let ruby_result = parse_source(ruby, "worker.rb", "ruby")?;
    assert_has_symbol(&ruby_result, "Worker", "class");
    assert_has_symbol(&ruby_result, "run", "method");
    assert!(
        ruby_result
            .imports
            .iter()
            .any(|import_| import_.raw_path.contains("json"))
    );
    assert_has_ref(&ruby_result, "BaseWorker", REF_KIND_IMPLEMENTS);
    assert!(ruby_result.refs.iter().any(|reference| reference.name == "helper"));

    let bash = br#"source "./lib.sh"

run() {
  helper
}
"#;
    let bash_result = parse_source(bash, "run.sh", "bash")?;
    assert_has_symbol(&bash_result, "run", "function");
    assert!(
        bash_result
            .imports
            .iter()
            .any(|import_| import_.raw_path.contains("./lib.sh"))
    );
    assert!(bash_result.refs.iter().any(|reference| reference.name == "helper"));

    let lua = br#"local base = require("base")

function run()
  helper()
end
"#;
    let lua_result = parse_source(lua, "run.lua", "lua")?;
    assert_has_symbol(&lua_result, "run", "function");
    assert!(
        lua_result
            .imports
            .iter()
            .any(|import_| import_.raw_path.contains("base"))
    );
    assert!(lua_result.refs.iter().any(|reference| reference.name == "helper"));

    let scala = br#"import service.BaseService

class Worker extends BaseService with Runnable

def runApp() = helper()
"#;
    let scala_result = parse_source(scala, "worker.scala", "scala")?;
    assert_has_symbol(&scala_result, "Worker", "class");
    assert_has_symbol(&scala_result, "runApp", "function");
    assert!(
        scala_result
            .imports
            .iter()
            .any(|import_| import_.raw_path.contains("service.BaseService"))
    );
    assert_has_ref(&scala_result, "BaseService", REF_KIND_IMPLEMENTS);
    assert_has_ref(&scala_result, "Runnable", REF_KIND_IMPLEMENTS);
    assert!(scala_result.refs.iter().any(|reference| reference.name == "helper"));

    Ok(())
}

fn assert_has_symbol<'a>(
    result: &'a ParseResult,
    name: &str,
    kind: &str,
) -> &'a sym::symbols::Symbol {
    result
        .symbols
        .iter()
        .find(|symbol| symbol.name == name && symbol.kind == kind)
        .unwrap_or_else(|| panic!("missing symbol {name} ({kind}) in {result:?}"))
}

fn assert_has_ref<'a>(result: &'a ParseResult, name: &str, kind: &str) -> &'a sym::symbols::Ref {
    result
        .refs
        .iter()
        .find(|reference| reference.name == name && reference.kind == kind)
        .unwrap_or_else(|| panic!("missing ref {name} ({kind}) in {result:?}"))
}
