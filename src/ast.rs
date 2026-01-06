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
    pub body: Vec<Statement>,
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

#[derive(Debug)]
pub enum Statement {
    VariableDeclaration {
        variable: Variable,
        value: Option<Expression>,
    },
    Assignment {
        target: String,
        value: Expression,
    },
    Return(Option<Expression>),
    Expression(Expression),
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    BinaryExpression {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    UnaryExpression {
        operator: UnaryOperator,
        right: Box<Expression>,
    },
    Call {
        function: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug)]
pub enum Literal {
    Int(i32),
    Bool(bool),
    String(String),
    Float(f32),
    Double(f64),
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Not,
    Neg,
}
