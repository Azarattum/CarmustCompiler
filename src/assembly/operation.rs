use crate::{ast::Primitive, error::assembly::AssemblyError, intermediate::Operation};

pub trait AssemblablePart {
    fn assemble<T: FnMut(bool) -> Result<String, AssemblyError>>(
        &self,
        allocate: T,
        datatype: Primitive,
        lhs: String,
        rhs: String,
    ) -> Result<Vec<String>, AssemblyError>;
    fn instruction(&self, datatype: Primitive) -> String;
    fn arity(&self) -> (usize, usize, bool);
}

impl AssemblablePart for Operation {
    fn assemble<T: FnMut(bool) -> Result<String, AssemblyError>>(
        &self,
        mut allocate: T,
        datatype: Primitive,
        lhs: String,
        rhs: String,
    ) -> Result<Vec<String>, AssemblyError> {
        Ok(match self {
            Self::Ldg => {
                let temp = allocate(true)?;
                vec![
                    format!("adrp {temp}, {rhs}"),
                    format!("ldr {temp}, [{temp}, {rhs}OFF]"),
                    format!("ldr {lhs}, [{temp}]"),
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
                    .map(|_| allocate(false))
                    .collect::<Result<Vec<_>, _>>()?;
                args.extend(([lhs, rhs])[0..operands].to_vec());
                if inverted {
                    args.reverse();
                }

                let instruction = format!("{} {}", self.instruction(datatype), args.join(", "));
                vec![instruction]
            }
        })
    }

    fn instruction(&self, datatype: Primitive) -> String {
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
            Self::Neg => "neg",
            Self::Ret => "ret",
            Self::FCvtZS => "fcvtzs",
            Self::SCvtF => "scvtf",
        };

        match (self, datatype) {
            (Self::Mov | Self::Add | Self::Mul | Self::Sub | Self::Div, Primitive::Float) => {
                format!("f{op}")
            }
            (Self::Div, Primitive::Byte | Primitive::Short | Primitive::Int | Primitive::Long) => {
                format!("s{op}")
            }
            (Self::Str | Self::Ldr, Primitive::Byte) => format!("{op}b"),
            (Self::Ldr, Primitive::Short) => format!("{op}sh"),
            (Self::Str, Primitive::Short) => format!("{op}h"),
            _ => op.to_owned(),
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
            Operation::Ret | Operation::Ldg => (0, 0, false),
            Operation::Str => (2, 0, true),
        }
    }
}
