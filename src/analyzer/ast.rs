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
    BinaryOp {
        op: char,
        lhs: Box<Expression<'a>>,
        rhs: Box<Expression<'a>>,
    },
    UnaryOp {
        op: char,
        operand: Box<Expression<'a>>,
    },
    Literal(i64),
    Identifier(&'a str),
}

#[derive(Debug)]
pub struct VariableDeclaration<'a> {
    pub datatype: DataType<'a>,
    pub name: &'a str,
    pub value: Option<Expression<'a>>,
}

#[derive(Debug)]
pub struct Assignment<'a> {
    variable: &'a str,
    value: Expression<'a>,
}

#[derive(Debug)]
pub struct ForLoop<'a> {
    initialization: VariableDeclaration<'a>,
    condition: Expression<'a>,
    increment: Assignment<'a>,
    body: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub struct FunctionDeclaration<'a> {
    datatype: DataType<'a>,
    name: &'a str,
    body: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub enum Statement<'a> {
    VariableDeclaration(VariableDeclaration<'a>),
    FunctionDeclaration(Vec<Statement<'a>>),
    Assignment(Assignment<'a>),
    Expression(Expression<'a>),
    ForLoop(ForLoop<'a>),
    Return(i32),
    Empty,
}

#[derive(Debug)]
pub enum GlobalStatement<'a> {
    VariableDeclaration(VariableDeclaration<'a>),
    FunctionDeclaration(Vec<Statement<'a>>),
}

#[derive(Debug)]
pub struct Program<'a> {
    pub types: HashMap<&'a str, DataType<'a>>,
    pub globals: Vec<VariableDeclaration<'a>>,
    pub functions: Vec<Statement<'a>>,
}
