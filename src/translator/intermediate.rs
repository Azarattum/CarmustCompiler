use std::fmt::Debug;

use crate::{
    ast::{Data, Primitive},
    program::Program,
};

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
            Operand::Data(x) => write!(f, "{}", x),
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

impl Operand {
    pub fn datatype<'a>(
        &self,
        program: &'a Program,
        address: Option<usize>,
    ) -> Option<Primitive<'a>> {
        match self {
            Self::Address(x) if address.is_some() => {
                program.instructions[*x].datatype(program, Some(*x))
            }
            Self::Data(Data::Integer(_)) => Some(Primitive::Int),
            Self::Data(Data::Float(_)) => Some(Primitive::Float),
            Self::Data(Data::Short(_)) => Some(Primitive::Short),
            Self::Data(Data::Byte(_)) => Some(Primitive::Byte),
            Self::Data(Data::Long(_)) => Some(Primitive::Long),
            Self::Temp => {
                let address = address?;
                let position = program.instructions.iter().position(|x| {
                    x.operand1 == Operand::Address(address)
                        || x.operand2 == Operand::Address(address)
                })?;
                program.instructions[position].datatype(program, None)
            }
            Self::Identifier(x) => program
                .locals
                .get(x)
                .or_else(|| program.globals.get(x).and_then(|x| Some(&x.0)))
                .and_then(|x| Some(x.primitive())),
            _ => None,
        }
    }
}

impl Instruction {
    pub fn datatype<'a>(
        &self,
        program: &'a Program,
        address: Option<usize>,
    ) -> Option<Primitive<'a>> {
        let type1 = self.operand1.datatype(program, address);
        let type2 = self.operand2.datatype(program, address);

        match (type1, type2) {
            (Some(x), Some(y)) if x == y => return Some(x),
            (Some(x), None) => return Some(x),
            (None, Some(x)) => return Some(x),
            _ => None,
        }
    }
}
