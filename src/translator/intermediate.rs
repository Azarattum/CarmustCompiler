use std::fmt::Debug;

#[derive(Debug)]
pub enum Operation {
    Str,
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

#[derive(Clone)]
pub enum Operand<'a> {
    Identifier(&'a str),
    Literal(i64),
    Address(usize),
    Asm(&'a str),
    Temp,
    None,
}

pub struct Instruction<'a> {
    pub operation: Operation,
    pub operand1: Operand<'a>,
    pub operand2: Operand<'a>,
}

impl Debug for Operand<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Address(x) => write!(f, "@{}", x),
            Operand::Identifier(x) => write!(f, "'{}'", x),
            Operand::Literal(x) => write!(f, "{}", x),
            Operand::Temp => write!(f, "@"),
            Operand::None => write!(f, ""),
            Operand::Asm(x) => write!(f, "{}", x),
        }
    }
}

impl Debug for Instruction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} {:?}",
            self.operation, self.operand1, self.operand2
        )
    }
}
