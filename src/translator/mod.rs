pub mod intermediate;
pub mod program;

use crate::{
    ast::{
        Assignment, BinaryOperator, Data, Expression, Loop, Pointer, Primitive, Statement,
        UnaryOperator, Value, Variable,
    },
    semantic::SemanticError,
};
use intermediate::{Operand, Operation, BYTE, ZERO};
use program::Program;
use std::cmp::max;

pub trait Translatable<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>>;
}

impl<'a> Translatable<'a> for Statement<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        match self {
            Self::Type(typedef) => program.define_type(&typedef.name, typedef.datatype),
            Self::Assignment(assignment) => assignment.translate(program),
            Self::Variable(variable) => variable.translate(program),
            Self::Loop(repetition) => repetition.translate(program),
            Self::Return(value) => {
                if program.toplevel() {
                    return Err(SemanticError {
                        message: "Return is not allowed on the top-level!".to_owned(),
                        token: None,
                    });
                }

                value.translate(program)?;
                let operand = program.cast(program.last(), Some(Primitive::Int));
                program.instruct(Operation::Ret, operand, Operand::None);
                Ok(())
            }
            Self::Function(function) => {
                if function.name != "main" {
                    return Err(SemanticError {
                        message: "Only 'main' function is supported by this implementation!"
                            .to_owned(),
                        token: Some(&function.name),
                    });
                }

                program.push_scope();
                function.body.translate(program)?;
                program.pop_scope();
                Ok(())
            }
            Self::Noop => Ok(()),
        }
    }
}

impl<'a> Translatable<'a> for Assignment<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        if program.toplevel() {
            return Err(SemanticError {
                message: "Assignments are not allowed on the top-level!".to_owned(),
                token: Some(self.name),
            });
        }

        self.value.translate(program)?;
        let identifier = program.infer_name(&self.name)?;
        let value = program.cast(program.last(), program.type_of(&identifier));
        program.instruct(
            Operation::Str,
            Operand::Identifier(program.infer_name(self.name)?),
            value,
        );
        Ok(())
    }
}

impl<'a> Translatable<'a> for Variable<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        match self.assignment {
            Some(Assignment {
                name,
                value: Expression::Value(Value::Data(data)),
            }) => program.define_variable(name, self.datatype, Some(data))?,
            Some(_) | None => {
                program.define_variable(&self.name, self.datatype, None)?;
            }
        }
        if !program.toplevel()
            && let Some(assignment) = self.assignment
        {
            assignment.translate(program)?;
        }

        Ok(())
    }
}

impl<'a> Translatable<'a> for Loop<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        if program.toplevel() {
            return Err(SemanticError {
                message: "Loops are not allowed on the top-level!".to_owned(),
                token: Some(self.initialization.name),
            });
        }
        program.push_scope();
        program.pop_scope();
        todo!()
    }
}

impl<'a> Translatable<'a> for Expression<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        match self {
            Self::Value(value) => {
                match value {
                    Value::Data(data) => {
                        program.instruct(Operation::Mov, Operand::Temp, Operand::Data(data));
                    }
                    Value::Pointer(
                        Pointer::Array(identifier, _) | Pointer::Identifier(identifier),
                    ) => {
                        program.instruct(
                            if program.is_global(identifier)? {
                                Operation::Ldg
                            } else {
                                Operation::Ldr
                            },
                            Operand::Temp,
                            Operand::Identifier(program.infer_name(identifier)?),
                        );
                    }
                };
            }
            Self::Unary { op, lhs } => {
                lhs.translate(program)?;
                match op {
                    UnaryOperator::Negation => {
                        program.instruct(Operation::Neg, program.last(), Operand::None);
                    }
                    UnaryOperator::Inversion => {
                        program.instruct(
                            Operation::Cmp,
                            program.last(),
                            Operand::Data(Data::Integer(0)),
                        );
                        program.instruct(Operation::CSet, Operand::Asm("eq"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                }
            }
            Self::Binary { op, lhs, rhs } => {
                lhs.translate(program)?;
                let operand1 = program.last();
                rhs.translate(program)?;
                let operand2 = program.last();

                let upcast = max(operand1.datatype(program), operand2.datatype(program));
                let operand1 = program.cast(operand1, upcast);
                let operand2 = program.cast(operand2, upcast);

                match op {
                    BinaryOperator::Addition => {
                        program.instruct(Operation::Add, operand1, operand2)
                    }
                    BinaryOperator::Subtraction => {
                        program.instruct(Operation::Sub, operand1, operand2)
                    }
                    BinaryOperator::Division => {
                        program.instruct(Operation::Div, operand1, operand2)
                    }
                    BinaryOperator::Multiplication => {
                        program.instruct(Operation::Mul, operand1, operand2)
                    }
                    BinaryOperator::BitwiseAnd => {
                        program.instruct(Operation::And, operand1, operand2)
                    }
                    BinaryOperator::BitwiseOr => {
                        program.instruct(Operation::Orr, operand1, operand2)
                    }
                    BinaryOperator::BitwiseXor => {
                        program.instruct(Operation::Eor, operand1, operand2)
                    }
                    BinaryOperator::Remainder => {
                        program.instruct(Operation::Div, operand1.clone(), operand2.clone());
                        program.instruct(Operation::Mul, operand2, program.last());
                        program.instruct(Operation::Sub, operand1, program.last());
                    }
                    BinaryOperator::LeftShift => {
                        program.instruct(Operation::Lsl, operand1, operand2);
                    }
                    BinaryOperator::RightShift => {
                        program.instruct(Operation::Asr, operand1, operand2);
                    }
                    BinaryOperator::Equal => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("eq"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::NotEqual => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("ne"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::Greater => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("gt"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::Less => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("lt"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::GreaterEqual => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("ge"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::LessEqual => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("le"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::And => {
                        program.instruct(Operation::And, operand1, operand2);
                        program.instruct(Operation::Cmp, program.last(), ZERO);
                        program.instruct(Operation::CSet, Operand::Asm("ne"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                    BinaryOperator::Or => {
                        program.instruct(Operation::Orr, operand1, operand2);
                        program.instruct(Operation::Cmp, program.last(), ZERO);
                        program.instruct(Operation::CSet, Operand::Asm("ne"), Operand::None);
                        program.instruct(Operation::And, program.last(), BYTE);
                    }
                }
            }
        };
        Ok(())
    }
}

impl<'a> Translatable<'a> for Vec<Statement<'a>> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        for statement in self {
            statement.translate(program)?;
        }
        Ok(())
    }
}
