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

    pub fn generate(program: &Program) -> String {
        let mut generator = JavaGenerator::new();
        generator.create_program(program);
        generator.output
    }

    pub fn create_program(&mut self, program: &Program) {
        for class in &program.classes {
            self.create_class(class);
        }
    }

    pub fn create_class(&mut self, class: &Class) {
        self.create_line(&format!("class {} {{", class.name));
        self.indent += 1;

        for method in &class.methods {
            self.create_method(method);
        }

        self.indent -= 1;
        self.create_line("}");
    }

    pub fn create_method(&mut self, method: &Method) {
        let modifiers = java_modifier(&method.modifiers);
        let return_type = java_type(&method.return_type);
        let parameters = java_parameters(&method.parameters);
        self.create_line(&format!(
            "{}{} {} ({}){{",
            modifiers, return_type, method.name, parameters
        ));
        self.indent += 1;

        self.indent -= 1;
        self.create_line("}");
    }

    pub fn create_line(&mut self, line: &str) {
        self.output.push_str(&"    ".repeat(self.indent));
        self.output.push_str(line);
        self.output.push_str("\n");
    }
}

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
