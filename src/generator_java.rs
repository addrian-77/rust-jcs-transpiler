use crate::ast::*;

pub struct JavaGenerator {
    indent: usize,
    output: String,
}

impl JavaGenerator {
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
        }
    }

    /// Calling this function will continue calling child functions until the program is complete
    pub fn generate(program: &Program) -> String {
        let mut generator = JavaGenerator::new();
        // call the create_program
        generator.create_program(program);
        // return the output
        generator.output
    }

    /// This function will iterate through a program's classes and call further creator functions
    pub fn create_program(&mut self, program: &Program) {
        for class in &program.classes {
            self.create_class(class);
        }
    }

    /// This function itereates through a class's methods and call further creator functions
    pub fn create_class(&mut self, class: &Class) {
        // begin creating actual lines of code
        if class.uses_input {
            self.create_line("import java.util.Scanner;");
        }
        self.create_line(&format!("class {} {{", class.name));
        // indent
        self.indent += 1;

        // iterate methods, create them
        for method in &class.methods {
            self.create_method(method, class.uses_input);
        }

        // substract indent
        self.indent -= 1;
        // close the program
        self.create_line("}");
    }

    /// This function creates a method's body
    pub fn create_method(&mut self, method: &Method, input: bool) {
        let modifiers = java_modifier(&method.modifiers);
        let return_type = java_type(&method.return_type);
        let parameters = java_parameters(&method.parameters);
        self.create_line(&format!(
            "{}{} {}({}) {{",
            modifiers,
            return_type,
            match method.name.as_ref() {
                "Main" => "main".to_string(),
                _ => method.name.clone(),
            },
            match method.name.as_ref() {
                "Main" =>
                    "String[] args".to_string()
                        // add a space if we have more arguments
                        + if parameters.len() > 0 { ", " } else { "" }
                        + &parameters,
                _ => parameters,
            }
        ));
        self.indent += 1;
        if input {
            self.create_line("Scanner scanner = new Scanner(System.in);");
        }

        for statement in &method.body {
            self.create_statement(statement);
        }

        self.indent -= 1;
        self.create_line("}");
    }

    /// This function creates a line of code by applying an indent
    pub fn create_line(&mut self, line: &str) {
        self.output.push_str(&"    ".repeat(self.indent));
        self.output.push_str(line);
        self.output.push_str("\n");
    }

    fn create_statement(&mut self, stmt: &Statement) {
        match stmt {
            // create a variable declaration statement, TYPE VAR = VALUE
            Statement::VariableDeclaration { variable, value } => {
                let mut line = format!("{} {}", java_type(&variable.typ), variable.name);

                if let Some(expr) = value {
                    line.push_str(" = ");
                    line.push_str(&self.create_expression(expr));
                }

                line.push(';');
                self.create_line(&line);
            }
            // assignment statement, VAR = VALUE
            Statement::Assignment { target, value } => {
                let value_str: String = self.create_expression(value);
                self.create_line(&format!("{} = {};", target, value_str));
            }
            // generic expression
            Statement::Expression(expr) => {
                let expr_str = self.create_expression(expr);
                self.create_line(&format!("{};", expr_str));
            }
            // return statement, can be with parameter or not
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    let expr_str = self.create_expression(e);
                    self.create_line(&format!("return {};", expr_str));
                } else {
                    self.create_line("return;");
                }
            }
            // if statement
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                // create the condition
                let cond_str = self.create_expression(condition);
                self.create_line(&format!("if ({}) {{", cond_str));
                // indent for the "then" block
                self.indent += 1;
                // iterate statements
                for statement in then_body {
                    self.create_statement(statement);
                }
                self.indent -= 1;
                self.create_line("}");

                // else can be empty, if it is not, create just like the "then" block
                if !else_body.is_empty() {
                    self.create_line("else {");
                    self.indent += 1;
                    for statement in else_body {
                        self.create_statement(statement);
                    }
                    self.indent -= 1;
                    self.create_line("}");
                }
            }
            // while statement
            Statement::While { condition, body } => {
                // create the condition
                let cond_str = self.create_expression(condition);
                self.create_line(&format!("while ({}) {{", cond_str));

                // create the while block, indent
                self.indent += 1;
                for statement in body {
                    self.create_statement(statement);
                }
                self.indent -= 1;

                self.create_line("}");
            }
            // for statement
            Statement::For {
                initializer,
                condition,
                increment,
                body,
            } => {
                // initializer can be missing, skip it if it is not present
                let init = initializer
                    .as_ref()
                    .map(|s| self.create_inline_statement(s))
                    .unwrap_or_default();
                // condition can also be missing
                let cond = condition
                    .as_ref()
                    .map(|cond| self.create_expression(cond))
                    .unwrap_or_default();
                // increment can also be missing
                let inc = increment
                    .as_ref()
                    .map(|s| self.create_inline_statement(s))
                    .unwrap_or_default();
                // create the initializer line
                self.create_line(&format!("for ({}; {}; {}) {{", init, cond, inc));
                // indent, create body
                self.indent += 1;
                for statement in body {
                    self.create_statement(statement);
                }
                self.indent -= 1;

                self.create_line("}");
            }
        }
    }
    /// This is a helper function that generates
    /// statements meant for using on the same line
    fn create_inline_statement(&mut self, statement: &Statement) -> String {
        match statement {
            // inline needed for variable declaration
            Statement::VariableDeclaration { variable, value } => {
                // type and variable name is mandatory
                let mut out = format!("{} {}", java_type(&variable.typ), variable.name);
                // the initial value is optional
                if let Some(expr) = value {
                    out.push_str(" = ");
                    out.push_str(&self.create_expression(expr));
                }

                out
            }

            Statement::Assignment { target, value } => {
                format!("{} = {}", target, self.create_expression(value))
            }
            // generic expression
            Statement::Expression(expr) => self.create_expression(expr),

            _ => String::new(),
        }
    }

    /// This function turns expressions into strings
    pub fn create_expression(&mut self, expr: &Expression) -> String {
        match expr {
            // the literals can safely be converted to strings using to_string
            Expression::Literal(lit) => match lit {
                Literal::Int(n) => n.to_string(),
                Literal::Bool(b) => b.to_string(),
                // strings already have the quote symbols, there is no need to add them here
                Literal::String(s) => format!("{}", s),
                Literal::Float(f) => f.to_string() + "f",
                Literal::Double(d) => d.to_string() + "d",
            },
            // just return the variable name
            Expression::Variable(name) => name.clone(),

            Expression::BinaryExpression {
                left,
                operator,
                right,
            } => format!(
                // use this function to create the left and right expressions
                // use the helper function to obtain the operator as a string
                "{} {} {}",
                self.create_expression(left),
                java_binary_operator(operator),
                self.create_expression(right)
            ),

            Expression::PrefixUnaryExpression { operator, right } => format!(
                // similar to binary expression, but we use the unary operator helper
                "{}{}",
                java_unary_operator(operator),
                self.create_expression(right)
            ),

            Expression::PostfixUnaryExpression { left, operator } => format!(
                // same as prefix, different order
                "{}{}",
                self.create_expression(left),
                java_unary_operator(operator)
            ),

            Expression::Call {
                function,
                arguments,
            } => {
                match function.as_str() {
                    // user output
                    "Console.WriteLine" => {
                        let args = arguments
                            .iter()
                            .map(|arg| self.create_expression(arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("System.out.println({})", args)
                    }

                    // user input, change the flag
                    "Console.ReadLine" => "scanner.nextLine()".to_string(),

                    // read and parse
                    "int.Parse" => "scanner.nextInt()".to_string(),

                    "double.Parse" => "scanner.nextDouble()".to_string(),

                    "bool.Parse" => "scanner.nextBoolean()".to_string(),

                    // anything else
                    _ => {
                        let args = arguments
                            .iter()
                            .map(|arg| self.create_expression(arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{}({})", function, args)
                    }
                }
            }
        }
    }
}

/// This functions turns a BinaryOperator into
/// the corresponding string
fn java_binary_operator(op: &BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::Add => "+",
        BinaryOperator::Sub => "-",
        BinaryOperator::Mul => "*",
        BinaryOperator::Div => "/",
        BinaryOperator::Eq => "==",
        BinaryOperator::Ne => "!=",
        BinaryOperator::Lt => "<",
        BinaryOperator::Gt => ">",
        BinaryOperator::Le => "<=",
        BinaryOperator::Ge => ">=",
        BinaryOperator::And => "&&",
        BinaryOperator::Or => "||",
    }
}

/// This functions turns an UnaryOperator into
/// the corresponding string
fn java_unary_operator(op: &UnaryOperator) -> &'static str {
    match op {
        UnaryOperator::Not => "!",
        UnaryOperator::Neg => "-",
        UnaryOperator::UAdd => "++",
        UnaryOperator::USub => "--",
    }
}

/// Helper function for parsing modifiers
pub fn java_modifier(modifiers: &Vec<Modifier>) -> String {
    let mut out = String::new();
    for modifier in modifiers {
        match modifier {
            Modifier::Public => out.push_str("public "),
            Modifier::Private => out.push_str("private "),
            Modifier::Static => out.push_str("static "),
            _ => (),
        }
    }
    out
}

/// Helper function for parsing types
pub fn java_type(typ: &Type) -> String {
    match typ {
        Type::Void => "void".to_string(),
        Type::Int => "int".to_string(),
        Type::Bool => "boolean".to_string(),
        Type::String => "String".to_string(),
        Type::Float => "float".to_string(),
        Type::Double => "double".to_string(),
        Type::Unknown => "Object".to_string(),
    }
}

/// Helper function for parsing parameters
pub fn java_parameters(parameters: &Vec<Variable>) -> String {
    let mut out = String::new();
    for parameter in parameters {
        out.push_str(&java_type(&parameter.typ));
        out.push(' ');
        out.push_str(&parameter.name);
        out.push_str(", ");
    }
    if !parameters.is_empty() {
        out.pop();
        out.pop();
    }
    out
}
