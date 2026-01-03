use tree_sitter::{self, Node};

use crate::ast::*;

pub fn find_classes(node: Node, source: &str, classes: &mut Vec<Class>) {
    if node.kind() == "class_declaration" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = source[name_node.byte_range()].to_string();
            let mut methods = Vec::new();
            find_methods(node, source, &mut methods);
            classes.push(Class { name, methods });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_classes(child, source, classes);
    }
}

pub fn find_methods(node: Node, source: &str, methods: &mut Vec<Method>) {
    if node.kind() == "method_declaration" {
        // let mut cursor = node.walk();
        // for child in node.children(&mut cursor) {
        //     println!("kind? {}", child.);
        // }
        // panic!();
        let name_node = node.child_by_field_name("name").unwrap();
        // let type_node = node.child_by_field_name("block").unwrap();

        // let return_type = source[type_node.byte_range()].to_string();
        let name = source[name_node.byte_range()].to_string();
        // println!("found method type {return_type}");
        methods.push(Method { name })
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_methods(child, source, methods);
    }
}

// debug functions

pub fn print_tree(node: Node, indent: usize) {
    let indent_str = " ".repeat(indent);
    println!("{}{}", indent_str, node.kind());

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, indent);
    }
}

pub fn find_everything(node: Node, source: &str) {
    if let Some(name_node) = node.child_by_field_name("name") {
        let name = source[name_node.byte_range()].to_string();
        print!("name: {name} ");
    }
    if let Some(type_node) = node.child_by_field_name("type") {
        let typ = source[type_node.byte_range()].to_string();
        print!("type: {typ}\n");
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_everything(child, source);
    }
}
