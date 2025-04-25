use crate::{ast::Primitive, error::assembly::AssemblyError, intermediate::Operation};

pub trait AssemblablePart {
    fn assemble<T: FnMut(bool, Option<Primitive>) -> Result<String, AssemblyError>>(
        &self,
        allocate: T,
        datatype: Option<Primitive>,
        lhs: String,
        rhs: String,
    ) -> Result<Vec<String>, AssemblyError>;
    fn instruction(&self, datatype: Option<Primitive>) -> Result<String, AssemblyError>;
    fn arity(&self) -> (usize, usize, bool);
}

impl AssemblablePart for Operation {
    fn assemble<T: FnMut(bool, Option<Primitive>) -> Result<String, AssemblyError>>(
        &self,
        mut allocate: T,
        datatype: Option<Primitive>,
        lhs: String,
        rhs: String,
    ) -> Result<Vec<String>, AssemblyError> {
        Ok(match self {
            // Weird hack to pass immediate constants refer to:
            //   https://stackoverflow.com/questions/64608307/how-do-i-move-a-floating-point-constant-into-an-fp-register
            Operation::Mov if rhs.starts_with("#") && datatype == Some(Primitive::Float) => {
                let temp = allocate(true, Some(Primitive::Int))?;
                vec![format!("mov {temp}, {rhs}"), format!("fmov {lhs}, {temp}")]
            }
            Operation::Mov if rhs.starts_with("=") => {
                vec![format!("ldr {lhs}, {rhs}")]
            }
            Operation::Lbl => vec![format!("{}:", lhs)],
            Operation::Ldg => {
                let temp = allocate(true, Some(Primitive::Long))?;
                let (identifier, offset) = rhs.split_once("@").ok_or(AssemblyError {
                    message: format!("Operand on global load instruction is invalid: {rhs}"),
                })?;

                vec![
                    format!("adrp {temp}, {identifier}@GOTPAGE"),
                    format!("ldr {temp}, [{temp}, {identifier}@GOTPAGEOFF]"),
                    format!("ldr {lhs}, [{temp}, {offset}]"),
                ]
            }
            Operation::Stg => {
                let temp = allocate(true, Some(Primitive::Long))?;
                let (identifier, offset) = lhs.split_once("@").ok_or(AssemblyError {
                    message: format!("Operand on global load instruction is invalid: {lhs}"),
                })?;

                vec![
                    format!("adrp {temp}, {identifier}@PAGE"),
                    format!("str {rhs}, [{temp}, {offset}]"),
                ]
            }
            Operation::Ret => vec![
                if lhs != "w0" && lhs != "x0" && lhs != "s0" {
                    Some(format!("mov w0, {lhs}"))
                } else {
                    None
                },
                Some(format!("ret")),
            ]
            .into_iter()
            .flatten()
            .collect(),
            _ => {
                let (operands, extra, inverted) = self.arity();
                let mut args = (0..extra)
                    .map(|_| allocate(false, datatype))
                    .collect::<Result<Vec<_>, _>>()?;
                args.extend(([lhs, rhs])[0..operands].to_vec());
                if inverted {
                    args.reverse();
                }

                let instruction = format!("{} {}", self.instruction(datatype)?, args.join(", "));
                vec![instruction]
            }
        })
    }

    fn instruction(&self, datatype: Option<Primitive>) -> Result<String, AssemblyError> {
        let op = match self {
            Self::Mov => "mov",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::And => "and",
            Self::Orr => "orr",
            Self::Eor => "eor",
            Self::Asr => "asr",
            Self::Lsl => "lsl",
            Self::CSet => "cset",
            Self::Cmp => "cmp",
            Self::Str => "str",
            Self::Ldr => "ldr",
            Self::Ldg => "ldg",
            Self::Stg => "stg",
            Self::Neg => "neg",
            Self::Ret => "ret",
            Self::FCvtZS => "fcvtzs",
            Self::SCvtF => "scvtf",
            Self::Lbl => "",
            Self::B => "b",
            Self::BEq => "b.eq",
        };

        match (self, datatype) {
            (
                Self::Mov | Self::Add | Self::Mul | Self::Sub | Self::Div | Self::Str | Self::Ldr,
                None,
            ) => Err(AssemblyError {
                message: format!("Instruction {self:?} requires a known datatype!"),
            }),
            (Self::Mov | Self::Add | Self::Mul | Self::Sub | Self::Div, Some(Primitive::Float)) => {
                Ok(format!("f{op}"))
            }
            (
                Self::Div,
                Some(Primitive::Byte | Primitive::Short | Primitive::Int | Primitive::Long),
            ) => Ok(format!("s{op}")),
            (Self::Str | Self::Ldr, Some(Primitive::Byte)) => Ok(format!("{op}b")),
            (Self::Ldr, Some(Primitive::Short)) => Ok(format!("{op}sh")),
            (Self::Str, Some(Primitive::Short)) => Ok(format!("{op}h")),
            _ => Ok(op.to_owned()),
        }
    }

    fn arity(&self) -> (usize, usize, bool) {
        match self {
            Operation::Add
            | Operation::Sub
            | Operation::Mul
            | Operation::Div
            | Operation::And
            | Operation::Orr
            | Operation::Eor
            | Operation::Asr
            | Operation::Lsl => (2, 1, false),
            Operation::Neg | Operation::CSet | Operation::FCvtZS | Operation::SCvtF => {
                (1, 1, false)
            }
            Operation::Cmp | Operation::Mov | Operation::Ldr => (2, 0, false),
            Operation::Lbl | Operation::B | Operation::BEq => (1, 0, false),
            Operation::Ret | Operation::Ldg | Operation::Stg => (0, 0, false),
            Operation::Str => (2, 0, true),
        }
    }
}
