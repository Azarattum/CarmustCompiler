use std::fmt::Debug;

use crate::ast::Data;

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
    SDiv,
    Mul,
    Neg,
    Cmp,
    CSet,
    And,
    Ret,
}

#[derive(Clone, PartialEq)]
pub enum Operand {
    Identifier(String),
    Asm(&'static str),
    Address(usize),
    Data(Data),
    Temp,
    None,
}

impl ToString for Data {
    fn to_string(&self) -> String {
        match self {
            Data::Float(x) => format!("{x:e}"),
            Data::Integer(x) => format!("{x}"),
        }
    }
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
            Operand::Identifier(x) => write!(f, "'{}'", x),
            Operand::Data(Data::Float(x)) => write!(f, "{:e}", x),
            Operand::Data(Data::Integer(x)) => write!(f, "{}", x),
            Operand::Temp => write!(f, "@"),
            Operand::None => write!(f, ""),
            Operand::Asm(x) => write!(f, "{}", x),
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
