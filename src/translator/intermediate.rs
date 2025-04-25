use crate::{
    ast::{Data, Primitive},
    program::Program,
};
use std::{cmp::min, fmt::Debug};

#[derive(Debug, PartialEq)]
pub enum Operation {
    Str,
    Ldr,
    Ldg,
    Mov,
    Add,
    Sub,
    Lsl,
    Asr,
    Orr,
    Eor,
    Div,
    Mul,
    Neg,
    Cmp,
    CSet,
    And,
    Ret,
    SCvtF,
    FCvtZS,
    Lbl,
    BEq,
    B,
}

#[derive(Clone, PartialEq)]
pub enum Operand {
    Identifier(String, usize),
    Asm(&'static str),
    Address(usize),
    Label(String),
    Data(Data),
    Temp,
    None,
}

pub struct Instruction {
    pub operation: Operation,
    pub operand1: Operand,
    pub operand2: Operand,
}

impl Debug for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Address(x) => write!(f, "@{}", x),
            Operand::Identifier(x, 0) => write!(f, "'{x}'"),
            Operand::Identifier(x, i) => write!(f, "'{x}[{i}]'"),
            Operand::Data(Data::Float(x)) => write!(f, "{:e}", x),
            Operand::Data(x) => write!(f, "{}", x),
            Operand::Temp => write!(f, "@"),
            Operand::None => write!(f, ""),
            Operand::Asm(x) => write!(f, "{}", x),
            Operand::Label(x) => write!(f, ":{}", x),
        }
    }
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?}",
            self.operation, self.operand1, self.operand2
        )
    }
}

pub const BYTE: Operand = Operand::Data(Data::Integer(255));
pub const ZERO: Operand = Operand::Data(Data::Integer(0));

impl Operand {
    pub fn datatype<'a>(&self, program: &'a Program) -> Option<Primitive> {
        match self {
            Self::Address(x) => program.instructions[*x].datatype(program),
            Self::Identifier(identifier, _) => program.type_of(identifier),
            Self::Data(Data::Integer(_)) => Some(Primitive::Int),
            Self::Data(Data::Float(_)) => Some(Primitive::Float),
            Self::Data(Data::Short(_)) => Some(Primitive::Short),
            Self::Data(Data::Byte(_)) => Some(Primitive::Byte),
            Self::Data(Data::Long(_)) => Some(Primitive::Long),
            _ => None,
        }
    }
}

impl Instruction {
    pub fn datatype<'a>(&self, program: &'a Program) -> Option<Primitive> {
        match self.operation {
            Operation::SCvtF => return Some(Primitive::Float),
            Operation::FCvtZS => return Some(Primitive::Int),
            Operation::CSet => return Some(Primitive::Int),
            _ => (),
        }

        let type1 = self.operand1.datatype(program);
        let type2 = self.operand2.datatype(program);

        match (type1, type2) {
            (Some(x), Some(y)) if x == y => return Some(x),
            (Some(x), None) => return Some(x),
            (None, Some(x)) => return Some(x),
            _ => min(type1, type2), // downcast
        }
    }
}
