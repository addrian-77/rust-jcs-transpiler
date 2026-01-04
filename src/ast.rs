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
    pub return_type: Type,
    pub modifiers: Vec<Modifier>,
    pub parameters: Vec<Variable>,
}

#[derive(Debug)]
pub enum Type {
    Void,
    Int,
    Bool,
    String,
    Float,
    Double,
    Unknown,
}

#[derive(Debug)]
pub enum Modifier {
    Public,
    Private,
    Static,
    Unknown,
}

#[derive(Debug)]
pub struct Variable {
    pub typ: Type,
    pub name: String,
}
