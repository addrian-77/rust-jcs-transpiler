#[derive(Debug)]
pub struct Program {
    pub classes: Vec<Class>,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
}
