use tree_sitter::{self, Node};

use crate::ast::*;

/// Recursively find all the classes of the given code
pub fn find_classes(node: Node, source: &str, classes: &mut Vec<Class>) {
    // extract class definition
    if node.kind() == "class_declaration" {
        if let Some(name_node) = node.child_by_field_name("name") {
            // obtain the name and methods
            let name = source[name_node.byte_range()].to_string();
            // extract methods here
            let mut methods = Vec::new();
            find_methods(node, source, &mut methods);
            // add to the classes vector
            classes.push(Class { name, methods });
        }
    }

    // find the rest of the classes recursively
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_classes(child, source, classes);
    }
}

/// Recursively find all the methods of a given class
pub fn find_methods(node: Node, source: &str, methods: &mut Vec<Method>) {
    // extract method definition
    if node.kind() == "method_declaration" {
        // obtain modifiers, save them in a &str vector
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

        // extract function details
        let name_node = node.child_by_field_name("name").unwrap();
        let type_node = node.child_by_field_name("returns").unwrap();

        // obtain parameters
        let mut parameters_raw: Vec<&str> = Vec::new();
        let parameters_node = node.child_by_field_name("parameters").unwrap();
        // iterate through the parameters_node's children
        for i in 0..parameters_node.child_count() {
            let child = parameters_node.child(i as u32).unwrap();
            // if this is a parameter node, extract the data
            if child.kind() == "parameter" {
                // println!("param {}", source[child.byte_range()].to_string());

                // save the parameter in this vector
                parameters_raw.push(&source[child.byte_range()]);
            }
        }

        // obtain the body node
        let body_node = node.child_by_field_name("body").unwrap();

        // search statements inside body
        let mut body_statements: Vec<Statement> = Vec::new();
        for i in 0..body_node.child_count() {
            let child = body_node.child(i as u32).unwrap();
            if child.kind() == "local_declaration_statement" {
                // this is a variable declaration
                body_statements.push(extract_var(child, source));
            }
            if child.kind() == "if_statement" {
                // extract the if statement
                body_statements.push(extract_if(child, source));
            }
            if child.kind() == "for_statement" {
                continue;
                // extract the for statement
                body_statements.push(extract_for(child, source));
            }
            if child.kind() == "while_statement" {
                continue;
                // extract the while statement
                body_statements.push(extract_while(child, source));
            }
            if child.kind() == "expression_statement" {
                // this can be anything
                let expr = child.child(0).unwrap();
                match expr.kind() {
                    // a special case is the assignment expression (a = a + b)
                    "assignment_expression" => {
                        body_statements.push(extract_assignment(expr, source))
                    }
                    // otherwise, treat it as a generic expression
                    _ => {
                        body_statements
                            .push(Statement::Expression(extract_expression(child, source)));
                    }
                }
            }
        }

        // parse the modifiers
        let modifiers = match_cs_modifiers(modifiers_raw);
        // extract the name
        let name = source[name_node.byte_range()].to_string();
        // extract the return type
        let return_type = source[type_node.byte_range()].to_string();
        // parse the parameters
        let parameters = match_cs_parameters(parameters_raw);

        // add to the methods vector
        methods.push(Method {
            name,
            return_type: match_cs_type(&return_type),
            modifiers,
            parameters,
            body: body_statements,
        })
    }

    // iterate the children, extract other methods recursively
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_methods(child, source, methods);
    }
}

/// Helper function for parsing a function or variable's type
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

/// Helper function for parsing function modifiers
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

/// Helper function for parsing variables
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

/// This function parses the variable_declaration statement
pub fn extract_var(node: Node, source: &str) -> Statement {
    let mut cursor = node.walk();
    // find the declaration node, this node will contain the other needed data
    let declaration_node = node
        .children(&mut cursor)
        .find(|n| n.kind() == "variable_declaration")
        .expect("Expected variable declaration");

    cursor = declaration_node.walk();
    // the type node will be used to parse a type
    let type_node = declaration_node
        .children(&mut cursor)
        .find(|n| n.kind() == "predefined_type")
        .expect("Expected type");

    // parse the type, save it here
    let typ = match_cs_type(&source[type_node.byte_range()]);

    cursor = declaration_node.walk();
    // the declarator node contains the identifier and value
    let declarator_node = declaration_node
        .children(&mut cursor)
        .find(|n| n.kind() == "variable_declarator")
        .expect("Expected identifier");

    cursor = declarator_node.walk();
    // find the identifier node
    let identifier_node = declarator_node
        .children(&mut cursor)
        .find(|n| n.kind() == "identifier")
        .expect("Expected identifier");

    // parse the name using the identifier node
    let name = source[identifier_node.byte_range()].to_string();

    // this will be used for finding the value later
    let literal_lookup = match typ {
        Type::Int => "integer_literal",
        Type::Bool => "boolean_literal",
        Type::String => "string_literal",
        Type::Float => "real_literal",
        Type::Double => "real_literal",
        _ => "",
    };

    // we will need both a literal and a value
    // the variable can be uninitialized, so we'll use value = None in that case
    let mut literal = Some(Literal::Int(0));
    let mut value: Option<Expression> = None;

    cursor = declarator_node.walk();
    // search the declarator node for the correct literal type
    if let Some(literal_node) = declarator_node
        .children(&mut cursor)
        .find(|n| n.kind() == literal_lookup)
    {
        // println!("inside?");

        // s contains the value of the literal
        let mut s = source[literal_node.byte_range()].to_string();
        // based on the type, parse the literal differently
        literal = Some(match typ {
            // parse an i32
            Type::Int => Literal::Int(s.parse::<i32>().unwrap()),
            // we can have true or false here
            Type::Bool => {
                if s.to_lowercase() == "true" {
                    Literal::Bool(true)
                } else {
                    Literal::Bool(false)
                }
            }
            // just use the string here
            Type::String => Literal::String(s),
            // remove the last character (if present), then parse as f32
            Type::Float => {
                if s.ends_with("f") || s.ends_with("F") {
                    s.pop();
                }
                Literal::Float(s.parse::<f32>().unwrap())
            }
            // same as float, but parse at f64
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
        // we can also have a binary expression when declaring a variable
        if let Some(binary_expr_node) = declarator_node
            .children(&mut cursor)
            .find(|n| n.kind() == "binary_expression")
        {
            // extract the binary expression using the helper
            value = Some(extract_binary_expression(binary_expr_node, source));
            // no literal in this case
            literal = None;
        } else {
            println!("uninitialized var");
        }
    }

    // construct the value here, if we don't have a literal, the value will be None
    match literal {
        Some(l) => value = Some(Expression::Literal(l)),
        _ => (),
    }

    // return the variable statement
    Statement::VariableDeclaration {
        variable: Variable { typ, name },
        value,
    }
}

/// This function extracts a binary expression
pub fn extract_binary_expression(node: Node, source: &str) -> Expression {
    // obtain the left node
    let left_node = node
        .child_by_field_name("left")
        .expect("binary_expression missing left");

    // obtain the right node
    let right_node = node
        .child_by_field_name("right")
        .expect("binary_expression missing right");

    // obtain the operator
    let operator_node = node
        .child_by_field_name("operator")
        .expect("binary_expression missing operator");

    // extract the left expression, put it in a Box
    let left = Box::new(extract_expression(left_node, source));
    // extract the right expression, put it in a Box
    let right = Box::new(extract_expression(right_node, source));

    // parse the operator accordingly
    let operator = match &source[operator_node.byte_range()] {
        "+" => BinaryOperator::Add,
        "-" => BinaryOperator::Sub,
        "*" => BinaryOperator::Mul,
        "/" => BinaryOperator::Div,
        "==" => BinaryOperator::Eq,
        "!=" => BinaryOperator::Ne,
        "<" => BinaryOperator::Lt,
        ">" => BinaryOperator::Gt,
        "<=" => BinaryOperator::Le,
        ">=" => BinaryOperator::Ge,
        "&&" => BinaryOperator::And,
        "||" => BinaryOperator::Or,
        _ => panic!("Unknown operator"),
    };

    // return the binary expression
    Expression::BinaryExpression {
        left,
        operator,
        right,
    }
}

/// This function parses unary expressions
pub fn extract_unary_expression(node: Node, source: &str) -> Expression {
    let mut cursor = node.walk();
    let mut children = node.children(&mut cursor);

    // unary expressions always have the same structure
    // the first node will always be the operator
    let operator_node = children.next().expect("Expected unary operator");
    // the following node will always be the operand
    let operand_node = children.next().expect("Expected unary operand");

    // parse the operator
    let operator = match &source[operator_node.byte_range()] {
        "!" => UnaryOperator::Not,
        "-" => UnaryOperator::Neg,
        _ => panic!("Unsupported unary operator"),
    };

    // parse the right of the expression
    let right = Box::new(extract_expression(operand_node, source));

    // return the unary expression
    Expression::UnaryExpression { operator, right }
}

/// This function parses if expressions
pub fn extract_if(node: Node, source: &str) -> Statement {
    // find the condition node
    let condition_node = node
        .child_by_field_name("condition")
        .expect("if_statement missing condition");

    // parse the condition as an expression
    let condition = extract_expression(condition_node, source);

    // find the then body of the if statement
    let then_block_node = node
        .child_by_field_name("consequence")
        .expect("if_statement missing consequence");

    // parse the then body using extract_block
    let then_body = extract_block(then_block_node, source);

    // the else block is optional
    let else_body = if let Some(alt_node) = node.child_by_field_name("alternative") {
        match alt_node.kind() {
            // we can encounter another block
            "block" => extract_block(alt_node, source),
            // or another if statement
            "if_statement" => vec![extract_if(alt_node, source)], // else-if
            _ => Vec::new(),
        }
    } else {
        // no else, emptty statement vector
        Vec::new()
    };

    // return the parsed if statement
    Statement::If {
        condition,
        then_body,
        else_body,
    }
}

pub fn extract_for(node: Node, source: &str) -> Statement {
    todo!();
}
pub fn extract_while(node: Node, source: &str) -> Statement {
    todo!();
}
pub fn extract_expression(node: Node, source: &str) -> Expression {
    // call different functions depending of the node kind
    match node.kind() {
        "expression_statement" => extract_expression(node.child(0).unwrap(), source),
        "binary_expression" => extract_binary_expression(node, source),
        "prefix_unary_expression" => extract_unary_expression(node, source),
        "invocation_expression" => extract_call_expression(node, source),

        // parse as i32
        "integer_literal" => Expression::Literal(Literal::Int(
            source[node.byte_range()].parse::<i32>().unwrap(),
        )),

        // here we can parse either as float (f32) or double (f64)
        "real_literal" => {
            let mut s = source[node.byte_range()].to_string();
            if s.ends_with("f") {
                s.pop();
                Expression::Literal(Literal::Float(
                    source[node.byte_range()].parse::<f32>().unwrap(),
                ))
            } else if s.ends_with("d") {
                s.pop();
                Expression::Literal(Literal::Double(s.parse::<f64>().unwrap()))
            } else {
                Expression::Literal(Literal::Float(s.parse::<f32>().unwrap()))
            }
        }

        // here we can have "true" or "false"
        "boolean_literal" => {
            let s = &source[node.byte_range()];
            if s == "true" {
                Expression::Literal(Literal::Bool(true))
            } else {
                Expression::Literal(Literal::Bool(false))
            }
        }
        "string_literal" => {
            Expression::Literal(Literal::String(source[node.byte_range()].to_string()))
        }
        // identifier expression, use the variable's name
        "identifier" => Expression::Variable(source[node.byte_range()].to_string()),
        _ => panic!("Unsupported expression: {}", node.kind()),
    }
}

/// This functions parses the assignment statements
pub fn extract_assignment(node: Node, source: &str) -> Statement {
    // extract the left node
    let left_node = node
        .child_by_field_name("left")
        .expect("assignment missing left");

    // extract the right node
    let right_node = node
        .child_by_field_name("right")
        .expect("assignment missing right");

    // left node should always be an identifier
    let target = match left_node.kind() {
        "identifier" => source[left_node.byte_range()].to_string(),
        _ => panic!("Unsupported assignment target"),
    };

    // the right of the identifier is an expression, extract it
    let value = extract_expression(right_node, source);

    // return the statement
    Statement::Assignment { target, value }
}

/// This function parses call expressions
pub fn extract_call_expression(node: Node, source: &str) -> Expression {
    // function name
    let function_node = node
        .child_by_field_name("function")
        .expect("invocation_expression missing function");

    // obtain the name using the helper
    let function = extract_function_name(function_node, source);

    // arguments
    let mut arguments = Vec::new();
    let args_node = node
        .child_by_field_name("arguments")
        .expect("invocation_expression missing arguments");

    let mut cursor = args_node.walk();
    // iterate through the children
    for child in args_node.children(&mut cursor) {
        if child.kind() == "argument" {
            let expr_node = child.child(0).unwrap();
            // parse the arguments as expressions
            arguments.push(extract_expression(expr_node, source));
        }
    }

    // return the call expression
    Expression::Call {
        function,
        arguments,
    }
}

/// This is a helper function that extracts function names
fn extract_function_name(node: Node, source: &str) -> String {
    match node.kind() {
        // simple function, just return the string
        "identifier" => source[node.byte_range()].to_string(),

        // compound function (Console.WriteLine)
        "member_access_expression" => {
            let left = node
                .child_by_field_name("expression")
                .expect("member access missing expression");

            let right = node
                .child_by_field_name("name")
                .expect("member access missing name");

            // extract the name recursively
            format!(
                "{}.{}",
                extract_function_name(left, source),
                extract_function_name(right, source)
            )
        }

        _ => panic!("Unsupported function node: {}", node.kind()),
    }
}

/// This function parses blocks of code
fn extract_block(block_node: Node, source: &str) -> Vec<Statement> {
    // create a new vector of statements
    let mut statements = Vec::new();
    let mut cursor = block_node.walk();

    // iterate the block node's children
    for child in block_node.children(&mut cursor) {
        // based on the kind, extract the statement accordingly
        match child.kind() {
            "local_declaration_statement" => {
                statements.push(extract_var(child, source));
            }
            "expression_statement" => {
                let expr = child.child(0).unwrap();
                match expr.kind() {
                    "assignment_expression" => statements.push(extract_assignment(expr, source)),
                    _ => {
                        statements.push(Statement::Expression(extract_expression(child, source)));
                    }
                }
            }
            "if_statement" => {
                statements.push(extract_if(child, source));
            }
            "while_statement" => {
                continue;
                statements.push(extract_while(child, source));
            }
            "for_statement" => {
                continue;
                statements.push(extract_for(child, source));
            }
            _ => {}
        }
    }

    statements
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
