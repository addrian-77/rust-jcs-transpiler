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
        let mut modifiers_raw: Vec<&str> = Vec::new();
        for i in 0..node.child_count() {
            let child = node.child(i as u32).unwrap();
            if child.kind() == "modifier" {
                // println!("modifier {}", source[child.byte_range()].to_string());
                modifiers_raw.push(&source[child.byte_range()]);
            } else {
                break;
            }
        }

        let name_node = node.child_by_field_name("name").unwrap();
        let type_node = node.child_by_field_name("returns").unwrap();

        let mut parameters_raw: Vec<&str> = Vec::new();
        let parameters_node = node.child_by_field_name("parameters").unwrap();
        for i in 0..parameters_node.child_count() {
            let child = parameters_node.child(i as u32).unwrap();
            if child.kind() == "parameter" {
                println!("param {}", source[child.byte_range()].to_string());
                parameters_raw.push(&source[child.byte_range()]);
            }
        }
        let body_node = node.child_by_field_name("body").unwrap();

        // search statements inside body
        let mut body_statements: Vec<Statement> = Vec::new();
        for i in 0..body_node.child_count() {
            let child = body_node.child(i as u32).unwrap();
            if child.kind() == "local_declaration_statement" {
                body_statements.push(extract_var(child, source));
            }
            if child.kind() == "if_statement" {
                body_statements.push(extract_if(child, source));
            }
            if child.kind() == "for_statement" {
                body_statements.push(extract_for(child, source));
            }
            if child.kind() == "while_statement" {
                body_statements.push(extract_while(child, source));
            }
            if child.kind() == "expression_statement" {
                body_statements.push(extract_expression(child, source));
            }
        }

        let modifiers = match_cs_modifiers(modifiers_raw);
        let name = source[name_node.byte_range()].to_string();
        let return_type = source[type_node.byte_range()].to_string();
        let parameters = match_cs_parameters(parameters_raw);

        methods.push(Method {
            name,
            return_type: match_cs_type(&return_type),
            modifiers,
            parameters,
            body: body_statements,
        })
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_methods(child, source, methods);
    }
}

pub fn match_cs_type(s: &str) -> Type {
    match s {
        "void" => Type::Void,
        "int" => Type::Int,
        "bool" => Type::Bool,
        "string" => Type::String,
        "float" => Type::Float,
        "double" => Type::Double,
        _ => Type::Unknown,
    }
}

pub fn match_cs_modifiers(modifiers: Vec<&str>) -> Vec<Modifier> {
    let mut out: Vec<Modifier> = Vec::new();
    for modifier in modifiers {
        match modifier {
            "public" => out.push(Modifier::Public),
            "private" => out.push(Modifier::Private),
            "static" => out.push(Modifier::Static),
            _ => out.push(Modifier::Unknown),
        }
    }
    out
}

pub fn match_cs_parameters(parameters: Vec<&str>) -> Vec<Variable> {
    let mut out: Vec<Variable> = Vec::new();
    for parameter in parameters {
        let param_raw = parameter.split_once(" ").unwrap();
        let typ = match_cs_type(param_raw.0);
        out.push(Variable {
            typ,
            name: param_raw.1.to_string(),
        })
    }
    out
}

pub fn extract_var(node: Node, source: &str) -> Statement {
    todo!();
}
pub fn extract_if(node: Node, source: &str) -> Statement {
    todo!();
}
pub fn extract_for(node: Node, source: &str) -> Statement {
    todo!();
}
pub fn extract_while(node: Node, source: &str) -> Statement {
    todo!();
}
pub fn extract_expression(node: Node, source: &str) -> Statement {
    todo!();
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
    // if node.kind() == "variable_declarator" {
    // for i in 0..node.child_count() {
    // let child = node.child(0 as u32).unwrap();
    let field = node.field_name_for_child(0 as u32);

    println!(
        "kind = {:<20} field = {:?} text = {}",
        node.kind(),
        field,
        source[node.byte_range()].to_string()
    );
    // }
    // }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_everything(child, source);
    }
}
