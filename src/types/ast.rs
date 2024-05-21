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

#[derive(Debug)]
pub enum Expression<'a> {
    Identifier(&'a str),
    Literal(i64),
    Binary {
        op: char,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    Unary {
        op: char,
        operand: Box<Expression<'a>>,
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
