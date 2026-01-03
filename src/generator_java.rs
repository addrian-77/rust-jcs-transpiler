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
        self.create_line(&format!("void {} {{", method.name));
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
