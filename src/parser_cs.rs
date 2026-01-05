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
    let mut cursor = node.walk();
    let declaration_node = node
        .children(&mut cursor)
        .find(|n| n.kind() == "variable_declaration")
        .expect("Expected variable declaration");

    cursor = declaration_node.walk();
    let type_node = declaration_node
        .children(&mut cursor)
        .find(|n| n.kind() == "predefined_type")
        .expect("Expected type");

    let typ = match_cs_type(&source[type_node.byte_range()]);

    cursor = declaration_node.walk();
    let declarator_node = declaration_node
        .children(&mut cursor)
        .find(|n| n.kind() == "variable_declarator")
        .expect("Expected identifier");

    cursor = declarator_node.walk();
    let identifier_node = declarator_node
        .children(&mut cursor)
        .find(|n| n.kind() == "identifier")
        .expect("Expected identifier");

    let name = source[identifier_node.byte_range()].to_string();

    let literal_lookup = match typ {
        Type::Int => "integer_literal",
        Type::Bool => "boolean_literal",
        Type::String => "string_literal",
        Type::Float => "real_literal",
        Type::Double => "real_literal",
        _ => "",
    };

    let mut literal = Some(Literal::Int(0));

    cursor = declarator_node.walk();
    if let Some(literal_node) = declarator_node
        .children(&mut cursor)
        .find(|n| n.kind() == literal_lookup)
    {
        println!("inside?");
        let mut s = source[literal_node.byte_range()].to_string();
        literal = Some(match typ {
            Type::Int => Literal::Int(s.parse::<i32>().unwrap()),
            Type::Bool => {
                if s.to_lowercase() == "true" {
                    Literal::Bool(true)
                } else {
                    Literal::Bool(false)
                }
            }
            Type::String => Literal::String(s),
            Type::Float => {
                if s.ends_with("f") || s.ends_with("F") {
                    s.pop();
                }
                Literal::Float(s.parse::<f32>().unwrap())
            }
            Type::Double => {
                if s.ends_with("d") || s.ends_with("D") {
                    s.pop();
                }
                Literal::Double(s.parse::<f64>().unwrap())
            }
            // this case is unreachable
            _ => Literal::Int(-1),
        });
    } else {
        println!("uninitialized var");
    }

    println!("typ {:#?}", typ);
    println!("name {:#?}", name);
    println!("literal {:#?}", literal);

    let mut value: Option<Expression> = None;

    match literal {
        Some(l) => value = Some(Expression::Literal(l)),
        _ => (),
    }
    let var_statement = Statement::VariableDeclaration {
        variable: Variable { typ, name },
        value,
    };
    var_statement
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

pub fn find_everything(node: Node, source: &str, indent: usize) {
    // if node.kind() == "variable_declarator" {
    // for i in 0..node.child_count() {
    // let child = node.child(0 as u32).unwrap();
    let field = node.field_name_for_child(0 as u32);

    println!(
        "{}kind = {:<20} field = {:?} text = {}",
        " ".repeat(indent).to_string(),
        node.kind(),
        field,
        source[node.byte_range()].to_string()
    );
    // }
    // }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_everything(child, source, indent + 1);
    }
}
