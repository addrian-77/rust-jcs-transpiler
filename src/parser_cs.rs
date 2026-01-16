use tree_sitter::{self, Node};

use crate::ast::*;

/// Recursively find all the classes of the given code
pub fn find_classes(node: Node, source: &str, classes: &mut Vec<Class>) {
    // extract class definition
    if node.kind() == "class_declaration" {
        let mut cursor = node.walk();
        if let Some(name_node) = node
            .children(&mut cursor)
            .find(|n| n.kind() == "identifier")
        {
            // obtain the name and methods
            let name = source[name_node.byte_range()].to_string();
            // extract methods here
            let mut methods = Vec::new();
            let mut uses_input: bool = false;
            find_methods(node, source, &mut methods, &mut uses_input);
            // add to the classes vector
            classes.push(Class {
                name,
                methods,
                uses_input,
            });
        }
    }

    // find the rest of the classes recursively
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_classes(child, source, classes);
    }
}

/// Recursively find all the methods of a given class
pub fn find_methods(node: Node, source: &str, methods: &mut Vec<Method>, uses_input: &mut bool) {
    // extract method definition
    if node.kind() == "method_declaration" {
        let mut cursor = node.walk();
        // obtain modifiers, save them in a &str vector
        let mut modifiers_raw: Vec<&str> = Vec::new();
        for i in 0..node.child_count() {
            let child = node.child(i as u32).unwrap();
            if child.kind() == "modifier" {
                modifiers_raw.push(&source[child.byte_range()]);
            } else {
                break;
            }
        }

        // extract function details
        let name_node = node
            .children(&mut cursor)
            .find(|n| n.kind() == "identifier")
            .expect("Expected variable declaration");
        let type_node = node
            .children(&mut cursor)
            .find(|n| n.kind() == "predefined_type")
            .expect("Expected variable declaration");

        // obtain parameters
        let mut parameters_raw: Vec<&str> = Vec::new();
        let parameters_node = node
            .children(&mut cursor)
            .find(|n| n.kind() == "parameter_list")
            .expect("Expected variable declaration");
        // iterate through the parameters_node's children
        for i in 0..parameters_node.child_count() {
            let child = parameters_node.child(i as u32).unwrap();
            // if this is a parameter node, extract the data
            if child.kind() == "parameter" {
                // save the parameter in this vector
                parameters_raw.push(&source[child.byte_range()]);
            }
        }

        // obtain the body node
        let body_node = node
            .children(&mut cursor)
            .find(|n| n.kind() == "block")
            .expect("Expected variable declaration");

        // search statements inside body
        let mut body_statements: Vec<Statement> = Vec::new();
        for i in 0..body_node.child_count() {
            let child = body_node.child(i as u32).unwrap();
            if child.kind() == "local_declaration_statement" {
                // this is a variable declaration
                body_statements.push(extract_var(child, source, uses_input));
            }
            if child.kind() == "if_statement" {
                // extract the if statement
                body_statements.push(extract_if(child, source, uses_input));
            }
            if child.kind() == "for_statement" {
                // extract the for statement
                body_statements.push(extract_for(child, source, uses_input));
            }
            if child.kind() == "while_statement" {
                // extract the while statement
                body_statements.push(extract_while(child, source, uses_input));
            }
            if child.kind() == "return_statement" {
                // extract the return statement
                body_statements.push(extract_return(child, source, uses_input));
            }
            if child.kind() == "expression_statement" {
                // this can be anything
                let expr = child.child(0).unwrap();
                match expr.kind() {
                    // a special case is the assignment expression (a = a + b)
                    "assignment_expression" => {
                        body_statements.push(extract_assignment(expr, source, uses_input))
                    }
                    // otherwise, treat it as a generic expression
                    _ => {
                        body_statements.push(Statement::Expression(extract_expression(
                            child, source, uses_input,
                        )));
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
        find_methods(child, source, methods, uses_input);
    }
}

/// This function parses the variable_declaration statement
pub fn extract_var(node: Node, source: &str, uses_input: &mut bool) -> Statement {
    let mut cursor = node.walk();
    let declaration_node = if node.kind() != "variable_declaration" {
        node.children(&mut cursor)
            .find(|n| n.kind() == "variable_declaration")
            .expect("Expected variable declaration")
    } else {
        node
    };

    // get type
    cursor = declaration_node.walk();
    let type_node = declaration_node
        .children(&mut cursor)
        .find(|n| n.kind() == "predefined_type")
        .expect("Expected type");
    let typ = match_cs_type(&source[type_node.byte_range()]);

    // get declarator
    cursor = declaration_node.walk();
    let declarator_node = declaration_node
        .children(&mut cursor)
        .find(|n| n.kind() == "variable_declarator")
        .expect("Expected declarator");

    // get variable name
    cursor = declarator_node.walk();
    let identifier_node = declarator_node
        .children(&mut cursor)
        .find(|n| n.kind() == "identifier")
        .expect("Expected identifier");
    let name = source[identifier_node.byte_range()].to_string();

    // find initializer (everything after `=`)
    let mut value: Option<Expression> = None;
    for child in declarator_node.children(&mut cursor) {
        match child.kind() {
            "integer_literal" | "real_literal" | "string_literal" | "boolean_literal" => {
                value = Some(extract_expression(child, source, uses_input));
                break;
            }
            "binary_expression"
            | "prefix_unary_expression"
            | "postfix_unary_expression"
            | "invocation_expression"
            | "member_access_expression" => {
                value = Some(extract_expression(child, source, uses_input));
                break;
            }
            _ => {}
        }
    }

    Statement::VariableDeclaration {
        variable: Variable { typ, name },
        value,
    }
}

/// This functions parses the assignment statements
pub fn extract_assignment(node: Node, source: &str, uses_input: &mut bool) -> Statement {
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
    let value = extract_expression(right_node, source, uses_input);

    // return the statement
    Statement::Assignment { target, value }
}

/// This function parses if expressions
pub fn extract_if(node: Node, source: &str, uses_input: &mut bool) -> Statement {
    // find the condition node
    let condition_node = node
        .child_by_field_name("condition")
        .expect("if_statement missing condition");

    // parse the condition as an expression
    let condition = extract_expression(condition_node, source, uses_input);

    // find the then body of the if statement
    let then_block_node = node
        .child_by_field_name("consequence")
        .expect("if_statement missing consequence");

    // parse the then body using extract_block
    let then_body = extract_block(then_block_node, source, uses_input);

    // the else block is optional
    let else_body = if let Some(alt_node) = node.child_by_field_name("alternative") {
        match alt_node.kind() {
            // we can encounter another block
            "block" => extract_block(alt_node, source, uses_input),
            // or another if statement
            "if_statement" => vec![extract_if(alt_node, source, uses_input)], // else-if
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

/// This function parses for statements
fn extract_for(node: Node, source: &str, uses_input: &mut bool) -> Statement {
    // the for nodes are always in the same order, initializer, condition, increment, body
    let mut cursor = node.walk();
    let children: Vec<Node> = node.named_children(&mut cursor).collect();

    let initializer = children.get(0).map(|n| {
        Box::new(match n.kind() {
            "variable_declaration" => extract_var(*n, source, uses_input),
            "assignment_expression" => extract_assignment(*n, source, uses_input),
            _ => panic!("Unsupported for initializer: {}", n.kind()),
        })
    });

    let condition = children
        .get(1)
        .map(|n| extract_expression(*n, source, uses_input));

    let increment = children.get(2).map(|n| {
        Box::new(Statement::Expression(extract_expression(
            *n, source, uses_input,
        )))
    });

    let body_node = children.get(3).expect("for loop missing body");
    let body = extract_block(*body_node, source, uses_input);

    Statement::For {
        initializer,
        condition,
        increment,
        body,
    }
}

/// This function parses while statements
pub fn extract_while(node: Node, source: &str, uses_input: &mut bool) -> Statement {
    // condition
    let condition_node = node
        .child_by_field_name("condition")
        .expect("while_statement missing condition");

    let condition = extract_expression(condition_node, source, uses_input);

    // body
    let mut cursor = node.walk();
    let body_node = node
        .children(&mut cursor)
        .find(|n| n.kind() == "block")
        .expect("Expected body node (while)");

    // parse the body using extract_block
    let body = extract_block(body_node, source, uses_input);

    // return the while statement
    Statement::While { condition, body }
}

pub fn extract_return(node: Node, source: &str, uses_input: &mut bool) -> Statement {
    // return_statement
    // children: "return", expression?, ";"

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        match child.kind() {
            // if we find an expression, extract it
            "binary_expression"
            | "prefix_unary_expression"
            | "postfix_unary_expression"
            | "invocation_expression"
            | "identifier"
            | "integer_literal"
            | "real_literal"
            | "boolean_literal"
            | "string_literal" => {
                return Statement::Return(Some(extract_expression(child, source, uses_input)));
            }
            _ => {}
        }
    }

    // no expression → `return;`
    Statement::Return(None)
}

pub fn extract_expression(node: Node, source: &str, uses_input: &mut bool) -> Expression {
    // call different functions depending of the node kind
    match node.kind() {
        "argument" => {
            let expr_node = node.named_child(0).unwrap();
            extract_expression(expr_node, source, uses_input)
        }
        "expression_statement" => extract_expression(node.child(0).unwrap(), source, uses_input),
        "binary_expression" => extract_binary_expression(node, source, uses_input),
        "prefix_unary_expression" => extract_unary_expression(node, source, true, uses_input),
        "postfix_unary_expression" => extract_unary_expression(node, source, false, uses_input),
        "invocation_expression" => extract_call_expression(node, source, uses_input),

        // parse as i32
        "integer_literal" => Expression::Literal(Literal::Int(
            source[node.byte_range()].parse::<i32>().unwrap(),
        )),

        // here we can parse either as float (f32) or double (f64)
        "real_literal" => {
            let mut s = source[node.byte_range()].to_string();
            if s.ends_with("f") {
                s.pop();
                Expression::Literal(Literal::Float(s.parse::<f32>().unwrap()))
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

/// This function extracts a binary expression
pub fn extract_binary_expression(node: Node, source: &str, uses_input: &mut bool) -> Expression {
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
    let left = Box::new(extract_expression(left_node, source, uses_input));
    // extract the right expression, put it in a Box
    let right = Box::new(extract_expression(right_node, source, uses_input));

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
pub fn extract_unary_expression(
    node: Node,
    source: &str,
    prefix: bool,
    uses_input: &mut bool,
) -> Expression {
    match prefix {
        true => {
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
            let right = Box::new(extract_expression(operand_node, source, uses_input));

            // return the unary expression
            Expression::PrefixUnaryExpression { operator, right }
        }
        false => {
            let mut cursor = node.walk();
            let mut children = node.children(&mut cursor);

            // unary expressions always have the same structure
            // the first node will always be the operator
            let operand_node = children.next().expect("Expected unary operator");
            // the following node will always be the operand
            let operator_node = children.next().expect("Expected unary operand");

            // parse the operator
            let operator = match &source[operator_node.byte_range()] {
                "++" => UnaryOperator::UAdd,
                "--" => UnaryOperator::USub,
                _ => panic!("Unsupported unary operator"),
            };

            // parse the left of the expression
            let left = Box::new(extract_expression(operand_node, source, uses_input));

            // return the unary expression
            Expression::PostfixUnaryExpression { left, operator }
        }
    }
}

/// This function parses call expressions
pub fn extract_call_expression(node: Node, source: &str, uses_input: &mut bool) -> Expression {
    // function name
    let function_node = node
        .child_by_field_name("function")
        .expect("invocation_expression missing function");

    // obtain the name using the helper
    let function = extract_function_name(function_node, source);
    if matches!(
        function.as_str(),
        "Console.ReadLine" | "int.Parse" | "double.Parse" | "bool.Parse"
    ) {
        *uses_input = true;
    }
    // arguments
    let mut arguments = Vec::new();
    let args_node = node
        .child_by_field_name("arguments")
        .expect("invocation_expression missing arguments");

    let mut cursor = args_node.walk();
    // iterate through the children
    for child in args_node.children(&mut cursor) {
        if child.kind() == "argument" {
            // parse the whole argument recursively
            arguments.push(extract_expression(child, source, uses_input));
        }
    }

    // return the call expression
    Expression::Call {
        function,
        arguments,
    }
}

/// This function parses blocks of code
fn extract_block(block_node: Node, source: &str, uses_input: &mut bool) -> Vec<Statement> {
    // create a new vector of statements
    let mut statements = Vec::new();
    let mut cursor = block_node.walk();

    // iterate the block node's children
    for child in block_node.children(&mut cursor) {
        // based on the kind, extract the statement accordingly
        match child.kind() {
            "local_declaration_statement" => {
                statements.push(extract_var(child, source, uses_input));
            }
            "expression_statement" => {
                let expr = child.child(0).unwrap();
                match expr.kind() {
                    "assignment_expression" => {
                        statements.push(extract_assignment(expr, source, uses_input))
                    }
                    _ => {
                        statements.push(Statement::Expression(extract_expression(
                            child, source, uses_input,
                        )));
                    }
                }
            }
            "if_statement" => {
                statements.push(extract_if(child, source, uses_input));
            }
            "while_statement" => {
                statements.push(extract_while(child, source, uses_input));
            }
            "for_statement" => {
                statements.push(extract_for(child, source, uses_input));
            }
            _ => {}
        }
    }

    statements
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

/// This is a helper function that extracts function names
fn extract_function_name(node: Node, source: &str) -> String {
    match node.kind() {
        // simple function or type like `int.Parse`
        "identifier" | "predefined_type" => source[node.byte_range()].to_string(),

        // compound function (Console.WriteLine, int.Parse, etc)
        "member_access_expression" => {
            let left = node
                .child_by_field_name("expression")
                .expect("member access missing expression");
            let right = node
                .child_by_field_name("name")
                .expect("member access missing name");
            format!(
                "{}.{}",
                extract_function_name(left, source),
                extract_function_name(right, source)
            )
        }

        _ => panic!("Unsupported function node: {}", node.kind()),
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

pub fn find_everything(node: Node, source: &str, indent: usize) {
    // if node.kind() == "variable_declarator" {
    // for i in 0..node.child_count() {
    // let child = node.child(0 as u32).unwrap();
    let mut cursor = node.walk();

    println!(
        "{}kind = {:<20} field = {:?} text = {}\n",
        " ".repeat(indent),
        node.kind(),
        node.field_name_for_child(0),
        source[node.byte_range()].to_string()
    );
    for child in node.children(&mut cursor) {
        find_everything(child, source, indent + 1);
    }
    // }
    // }
}
