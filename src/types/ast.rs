#[derive(Debug, Clone, Copy)]
pub enum Primitive<'a> {
    Int,
    Float,
    Short,
    Long,
    Char,
    Custom(&'a str),
}

impl Primitive<'_> {
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Long => Some(8),
            Self::Int => Some(4),
            Self::Float => Some(4),
            Self::Short => Some(2),
            Self::Char => Some(1),
            _ => None,
        }
    }

    pub fn floating(&self) -> bool {
        match self {
            Self::Float => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DataType<'a> {
    Primitive(Primitive<'a>),
    Array(Primitive<'a>, usize),
}

impl DataType<'_> {
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Primitive(primitive) => primitive.size(),
            Self::Array(primitive, size) if primitive.size().is_some() => {
                Some(size * primitive.size().unwrap())
            }
            _ => None,
        }
    }

    pub fn floating(&self) -> bool {
        let (Self::Primitive(primitive) | Self::Array(primitive, _)) = self;
        primitive.floating()
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Data {
    Integer(i64),
    Float(f64),
}

#[derive(Debug)]
pub enum Pointer<'a> {
    Identifier(&'a str),
    Array(&'a str, usize),
}

#[derive(Debug)]
pub enum Value<'a> {
    Data(Data),
    Pointer(Pointer<'a>),
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
pub struct Loop<'a> {
    pub initialization: Variable<'a>,
    pub condition: Expression<'a>,
    pub increment: Assignment<'a>,
    pub body: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub struct Variable<'a> {
    pub datatype: DataType<'a>,
    pub name: &'a str,
    pub assignment: Option<Assignment<'a>>,
}

#[derive(Debug)]
pub struct Assignment<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
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
pub enum Statement<'a> {
    Assignment(Assignment<'a>),
    Variable(Variable<'a>),
    Function(Function<'a>),
    Type(Type<'a>),
    Loop(Loop<'a>),
    Return(Expression<'a>),
}
