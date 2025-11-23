//! Integration tests for call detection across all languages.

use doctown_common::types::CallKind;
use doctown_ingest::{extract_calls, parse};

#[test]
fn test_call_detection_rust_comprehensive() {
    let code = r#"
use std::collections::HashMap;

fn main() {
    // Direct function calls
    foo();
    bar(1, 2);
    
    // Associated function calls
    let map = HashMap::new();
    let s = String::from("hello");
    
    // Method calls
    s.len();
    s.push_str(" world");
    
    // Chained method calls
    let result = vec![1, 2, 3]
        .iter()
        .map(|x| x * 2)
        .filter(|x| *x > 2)
        .collect::<Vec<_>>();
    
    // Self method calls
    let obj = MyStruct::new();
    obj.do_something();
}

struct MyStruct;

impl MyStruct {
    fn new() -> Self {
        Self
    }
    
    fn do_something(&self) {
        println!("doing something");
    }
}

fn foo() {}
fn bar(_a: i32, _b: i32) {}
"#;

    let tree = parse(code, doctown_common::Language::Rust).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::Rust);

    // Verify we found various types of calls
    assert!(calls.iter().any(|c| c.name.contains("foo")));
    assert!(calls.iter().any(|c| c.name.contains("bar")));
    assert!(calls.iter().any(|c| c.name.contains("new")));
    assert!(calls.iter().any(|c| c.name.contains("from")));
    assert!(calls.iter().any(|c| c.name.contains("len")));
    assert!(calls.iter().any(|c| c.name.contains("iter")));
    assert!(calls.iter().any(|c| c.name.contains("map")));
    assert!(calls.iter().any(|c| c.name.contains("collect")));
    
    // Verify we have both function and method calls
    assert!(calls.iter().any(|c| c.kind == CallKind::Function));
    assert!(calls.iter().any(|c| c.kind == CallKind::Method));
}

#[test]
fn test_call_detection_python_comprehensive() {
    let code = r#"
import json
from pathlib import Path

def main():
    # Direct function calls
    print("hello")
    foo()
    bar(1, 2)
    
    # Class instantiation
    obj = MyClass()
    obj2 = MyClass(arg1="value")
    
    # Method calls
    s = "hello world"
    s.upper()
    s.replace("world", "universe")
    
    # Chained method calls
    result = [1, 2, 3].copy()
    
    # Module function calls
    data = json.loads('{"key": "value"}')
    path = Path("/tmp/file.txt")
    path.exists()

class MyClass:
    def __init__(self, arg1=None):
        self.arg1 = arg1
    
    def method(self):
        pass

def foo():
    pass

def bar(a, b):
    pass
"#;

    let tree = parse(code, doctown_common::Language::Python).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::Python);

    // Verify we found various types of calls
    assert!(calls.iter().any(|c| c.name == "print"));
    assert!(calls.iter().any(|c| c.name == "foo"));
    assert!(calls.iter().any(|c| c.name == "bar"));
    assert!(calls.iter().any(|c| c.name == "MyClass"));
    assert!(calls.iter().any(|c| c.name.contains("upper")));
    assert!(calls.iter().any(|c| c.name.contains("replace")));
    assert!(calls.iter().any(|c| c.name.contains("loads")));
    assert!(calls.iter().any(|c| c.name.contains("exists")));
    
    // Verify we have both function, method, and constructor calls
    assert!(calls.iter().any(|c| c.kind == CallKind::Function));
    assert!(calls.iter().any(|c| c.kind == CallKind::Method));
    assert!(calls.iter().any(|c| c.kind == CallKind::Constructor));
}

#[test]
fn test_call_detection_typescript_comprehensive() {
    let code = r#"
import { readFile } from 'fs';

function main() {
    // Direct function calls
    console.log("hello");
    foo();
    bar(1, 2);
    
    // Constructor calls
    const obj = new MyClass();
    const obj2 = new MyClass("arg");
    
    // Method calls
    const arr = [1, 2, 3];
    arr.map(x => x * 2);
    arr.filter(x => x > 1);
    
    // Chained method calls
    const result = [1, 2, 3]
        .map(x => x * 2)
        .filter(x => x > 2)
        .reduce((acc, x) => acc + x, 0);
    
    // Module function calls
    readFile('/path/to/file', (err, data) => {
        if (err) throw err;
    });
}

class MyClass {
    constructor(arg?: string) {
        this.arg = arg;
    }
    
    method() {
        console.log("method called");
    }
}

function foo() {}
function bar(a: number, b: number) {}
"#;

    let tree = parse(code, doctown_common::Language::TypeScript).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::TypeScript);

    // Verify we found various types of calls
    assert!(calls.iter().any(|c| c.name.contains("log")));
    assert!(calls.iter().any(|c| c.name == "foo"));
    assert!(calls.iter().any(|c| c.name == "bar"));
    assert!(calls.iter().any(|c| c.name == "MyClass" && c.kind == CallKind::Constructor));
    assert!(calls.iter().any(|c| c.name.contains("map")));
    assert!(calls.iter().any(|c| c.name.contains("filter")));
    assert!(calls.iter().any(|c| c.name.contains("reduce")));
    
    // Verify we have all call types
    assert!(calls.iter().any(|c| c.kind == CallKind::Function));
    assert!(calls.iter().any(|c| c.kind == CallKind::Method));
    assert!(calls.iter().any(|c| c.kind == CallKind::Constructor));
}

#[test]
fn test_call_detection_javascript_comprehensive() {
    let code = r#"
const fs = require('fs');

function main() {
    // Direct function calls
    console.log("hello");
    foo();
    bar(1, 2);
    
    // Constructor calls
    const obj = new MyClass();
    
    // Method calls
    const arr = [1, 2, 3];
    arr.forEach(x => console.log(x));
    
    // Promise chains
    fetch('/api/data')
        .then(response => response.json())
        .then(data => console.log(data))
        .catch(err => console.error(err));
}

class MyClass {
    constructor() {
        this.value = 0;
    }
    
    increment() {
        this.value++;
    }
}

function foo() {}
function bar(a, b) {}
"#;

    let tree = parse(code, doctown_common::Language::JavaScript).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::JavaScript);

    // Verify we found various types of calls
    assert!(calls.iter().any(|c| c.name.contains("log")));
    assert!(calls.iter().any(|c| c.name == "foo"));
    assert!(calls.iter().any(|c| c.name == "bar"));
    assert!(calls.iter().any(|c| c.name == "MyClass" && c.kind == CallKind::Constructor));
    assert!(calls.iter().any(|c| c.name.contains("forEach")));
    assert!(calls.iter().any(|c| c.name == "fetch"));
    assert!(calls.iter().any(|c| c.name.contains("then")));
    
    // Verify we have all call types
    assert!(calls.iter().any(|c| c.kind == CallKind::Function));
    assert!(calls.iter().any(|c| c.kind == CallKind::Method));
    assert!(calls.iter().any(|c| c.kind == CallKind::Constructor));
}

#[test]
fn test_call_detection_go_comprehensive() {
    let code = r#"
package main

import (
    "fmt"
    "strings"
)

func main() {
    // Direct function calls
    fmt.Println("hello")
    foo()
    bar(1, 2)
    
    // Package function calls
    upper := strings.ToUpper("hello")
    _ = strings.Contains("hello", "ell")
    
    // Method calls
    s := MyStruct{}
    s.Method()
    
    // Pointer method calls
    ptr := &MyStruct{}
    ptr.Method()
    
    // Builtin functions
    length := len("hello")
    _ = length
}

type MyStruct struct {
    Value int
}

func (m *MyStruct) Method() {
    fmt.Println("method called")
}

func foo() {}
func bar(a int, b int) {}
"#;

    let tree = parse(code, doctown_common::Language::Go).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::Go);

    // Verify we found various types of calls
    assert!(calls.iter().any(|c| c.name.contains("Println")));
    assert!(calls.iter().any(|c| c.name == "foo"));
    assert!(calls.iter().any(|c| c.name == "bar"));
    assert!(calls.iter().any(|c| c.name.contains("ToUpper")));
    assert!(calls.iter().any(|c| c.name.contains("Contains")));
    assert!(calls.iter().any(|c| c.name.contains("Method")));
    assert!(calls.iter().any(|c| c.name == "len"));
    
    // Verify we have both function and method calls
    assert!(calls.iter().any(|c| c.kind == CallKind::Function));
    assert!(calls.iter().any(|c| c.kind == CallKind::Method));
}

#[test]
fn test_call_ranges_are_valid() {
    let code = r#"
fn main() {
    foo();
    bar(1, 2);
}
fn foo() {}
fn bar(_a: i32, _b: i32) {}
"#;

    let tree = parse(code, doctown_common::Language::Rust).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::Rust);

    // Verify all ranges are valid
    for call in &calls {
        assert!(call.range.start < call.range.end);
        assert!(call.range.end <= code.len());
        
        // Extract the text from the range and verify it's not empty
        let call_text = &code[call.range.start..call.range.end];
        assert!(!call_text.is_empty());
    }
}

#[test]
fn test_no_calls_in_empty_code() {
    let code = "";
    let tree = parse(code, doctown_common::Language::Rust).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::Rust);
    assert_eq!(calls.len(), 0);
}

#[test]
fn test_no_calls_in_comments() {
    let code = r#"
// foo();
/* bar(); */
fn main() {
    // This should not count: baz();
}
"#;

    let tree = parse(code, doctown_common::Language::Rust).unwrap();
    let calls = extract_calls(&tree, code, doctown_common::Language::Rust);
    
    // Should not find any calls (all are in comments)
    assert_eq!(calls.len(), 0);
}
