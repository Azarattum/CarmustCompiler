use std::collections::HashMap;

#[derive(Debug)]
pub enum Primitive<'a> {
    Int,
    Float,
    Short,
    Long,
    Char,
    Custom(&'a str),
}

#[derive(Debug)]
pub enum DataType<'a> {
    Primitive(Primitive<'a>),
    Array(Primitive<'a>, usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Binary(BinaryOperator),
    Unary(UnaryOperator),
    Group,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Remainder,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
    Or,
    And,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

impl BinaryOperator {
    pub fn precedence(&self) -> i32 {
        match *self {
            BinaryOperator::Division => 3,
            BinaryOperator::Multiplication => 3,
            BinaryOperator::Remainder => 3,
            BinaryOperator::Addition => 4,
            BinaryOperator::Subtraction => 4,
            BinaryOperator::LeftShift => 5,
            BinaryOperator::RightShift => 5,
            BinaryOperator::Greater => 6,
            BinaryOperator::Less => 6,
            BinaryOperator::GreaterEqual => 6,
            BinaryOperator::LessEqual => 6,
            BinaryOperator::Equal => 7,
            BinaryOperator::NotEqual => 7,
            BinaryOperator::BitwiseAnd => 8,
            BinaryOperator::BitwiseXor => 9,
            BinaryOperator::BitwiseOr => 10,
            BinaryOperator::And => 11,
            BinaryOperator::Or => 12,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Negation,
    Inversion,
}

#[derive(Debug)]
pub enum Value<'a> {
    Identifier(&'a str),
    Array(&'a str, usize),
    Integer(i64),
    Float(f64),
}

#[derive(Debug)]
pub enum Expression<'a> {
    Value(Value<'a>),
    Binary {
        op: BinaryOperator,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    Unary {
        op: UnaryOperator,
        lhs: Box<Expression<'a>>,
    },
}

#[derive(Debug)]
pub struct Assignment<'a> {
    variable: &'a str,
    value: Expression<'a>,
}

#[derive(Debug)]
pub struct ForLoop<'a> {
    initialization: Variable<'a>,
    condition: Expression<'a>,
    increment: Assignment<'a>,
    body: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub enum Statement<'a> {
    VariableDeclaration(Variable<'a>),
    FunctionDeclaration(Vec<Statement<'a>>),
    Assignment(Assignment<'a>),
    Expression(Expression<'a>),
    ForLoop(ForLoop<'a>),
    Return(i32),
}

#[derive(Debug)]
pub struct Variable<'a> {
    pub datatype: DataType<'a>,
    pub name: &'a str,
    pub value: Option<Expression<'a>>,
}

#[derive(Debug)]
pub struct Function<'a> {
    pub datatype: DataType<'a>,
    pub name: &'a str,
    pub body: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub struct Type<'a> {
    pub datatype: DataType<'a>,
    pub name: &'a str,
}

#[derive(Debug)]
pub enum Declaration<'a> {
    Variable(Variable<'a>),
    Function(Function<'a>),
    Type(Type<'a>),
}

#[derive(Debug)]
pub struct Program<'a> {
    pub types: HashMap<&'a str, DataType<'a>>,
    pub declarations: Vec<Declaration<'a>>,
}
