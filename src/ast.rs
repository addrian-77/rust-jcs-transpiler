// program is a vector of classes
#[derive(Debug)]
pub struct Program {
    pub classes: Vec<Class>,
}

// a class is a vector of methods
#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: Vec<Method>,
    pub uses_input: bool,
}

// here it gets a bit more complicated
#[derive(Debug)]
pub struct Method {
    pub name: String,              // method's name
    pub return_type: Type,         // the return type
    pub modifiers: Vec<Modifier>,  // modifiers, such as public, private
    pub parameters: Vec<Variable>, // parameters of the method
    pub body: Vec<Statement>,      // body, a vector of statements
}

// return and variable types
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

// method modifiers
#[derive(Debug)]
pub enum Modifier {
    Public,
    Private,
    Static,
    Unknown,
}

#[derive(Debug)]
pub struct Variable {
    pub typ: Type,    // type of the variable
    pub name: String, // name of variable
}

// we can have multiple statements in a code block
#[derive(Debug)]
pub enum Statement {
    VariableDeclaration {
        variable: Variable,        // the variable, containing type and name
        value: Option<Expression>, // the value, which can be a literal, or a boolean expression
    },
    Assignment {
        target: String,    // target of the assignment
        value: Expression, // value, a generic expression
    },
    If {
        condition: Expression,     // if condition
        then_body: Vec<Statement>, // then body, a code block made of statements
        else_body: Vec<Statement>, // same for else body
    },
    For {
        // initializer, condition and increment are optional,
        // we can have for (;;;)
        initializer: Option<Box<Statement>>, // the initializer can be a complex statement
        condition: Option<Expression>,       // condition is an expression
        increment: Option<Box<Statement>>,   // the increment can be a complex statement
        body: Vec<Statement>,                // body is a vector of statements
    },
    While {
        condition: Expression, // the while condition
        body: Vec<Statement>,  // body, a vec of statements
    },
    Return(Option<Expression>), // the return of a function
    Expression(Expression),     // a generic expression, handled as a statement
}

#[derive(Debug)]
pub enum Expression {
    Literal(Literal), // literals, values as int, float, double, string
    Variable(String), // variable name
    BinaryExpression {
        left: Box<Expression>,    // left side of the binary expression
        operator: BinaryOperator, // the operator
        right: Box<Expression>,   // right side
    },
    PrefixUnaryExpression {
        operator: UnaryOperator, // operator of the expression
        right: Box<Expression>,  // right side of expression
    },
    PostfixUnaryExpression {
        left: Box<Expression>,   // left side of expression
        operator: UnaryOperator, // operator
    },
    Call {
        function: String,           // function name
        arguments: Vec<Expression>, // vector of arguments
    },
}

#[derive(Debug)]
pub enum Literal {
    // literals represent values, such as int, float, string
    Int(i32),
    Bool(bool),
    String(String),
    Float(f32),
    Double(f64),
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Eq,  // ==
    Ne,  // !=
    Lt,  // <
    Gt,  // >
    Le,  // <=
    Ge,  // >=
    And, // &&
    Or,  // ||
}

#[derive(Debug)]
pub enum UnaryOperator {
    Not,  // !
    Neg,  // - (negative)
    UAdd, // ++
    USub, // --
}
