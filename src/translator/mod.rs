mod intermediate;
pub mod program;

use crate::{
    ast::{
        Assignment, BinaryOperator, Expression, Loop, Statement, UnaryOperator, Value, Variable,
    },
    semantic::SemanticError,
};
use intermediate::{Operand, Operation};
use program::Program;

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
                program.instruct(Operation::Ret, program.last(), Operand::None);
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
        if !program.is_defined(&self.name) {
            return Err(SemanticError {
                message: format!("'{}' is not defined!", self.name),
                token: Some(self.name),
            });
        }

        self.value.translate(program)?;
        program.instruct(
            Operation::Str,
            Operand::Identifier(self.name),
            program.last(),
        );
        Ok(())
    }
}

impl<'a> Translatable<'a> for Variable<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        match self.assignment {
            Some(Assignment {
                name,
                // TODO: support definitions for other types
                value: Expression::Value(Value::Integer(value)),
            }) => program.define_variable(name, self.datatype, Some(value)),
            Some(_) | None => {
                program.define_variable(&self.name, self.datatype, None)?;
                self.assignment.and_then(|x| Some(x.translate(program)));
                Ok(())
            }
        }
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
                let operand = match value {
                    Value::Integer(literal) => Operand::Literal(literal),
                    Value::Identifier(identifier) => {
                        if !program.is_defined(identifier) {
                            return Err(SemanticError {
                                message: format!("Usage of undefined variable '{}'", identifier),
                                token: Some(identifier),
                            });
                        }
                        Operand::Identifier(identifier)
                    }
                    _ => todo!(),
                };
                program.instruct(Operation::Mov, Operand::Temp, operand);
            }
            Self::Unary { op, lhs } => {
                lhs.translate(program)?;
                match op {
                    UnaryOperator::Negation => {
                        program.instruct(Operation::Neg, program.last(), Operand::None);
                    }
                    UnaryOperator::Inversion => {
                        program.instruct(Operation::Cmp, program.last(), Operand::Literal(0));
                        program.instruct(Operation::CSet, Operand::Asm("eq"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                }
            }
            Self::Binary { op, lhs, rhs } => {
                lhs.translate(program)?;
                let operand1 = program.last();
                rhs.translate(program)?;
                let operand2 = program.last();
                match op {
                    BinaryOperator::Addition => {
                        program.instruct(Operation::Add, operand1, operand2)
                    }
                    BinaryOperator::Subtraction => {
                        program.instruct(Operation::Sub, operand1, operand2)
                    }
                    BinaryOperator::Division => {
                        program.instruct(Operation::SDiv, operand1, operand2)
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
                        program.instruct(Operation::SDiv, operand1.clone(), operand2.clone());
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
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::NotEqual => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("ne"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::Greater => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("gt"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::Less => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("lt"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::GreaterEqual => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("ge"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::LessEqual => {
                        program.instruct(Operation::Cmp, operand1, operand2);
                        program.instruct(Operation::CSet, Operand::Asm("le"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::And => {
                        program.instruct(Operation::And, operand1, operand2);
                        program.instruct(Operation::Cmp, program.last(), Operand::Literal(0));
                        program.instruct(Operation::CSet, Operand::Asm("ne"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
                    }
                    BinaryOperator::Or => {
                        program.instruct(Operation::Orr, operand1, operand2);
                        program.instruct(Operation::Cmp, program.last(), Operand::Literal(0));
                        program.instruct(Operation::CSet, Operand::Asm("ne"), Operand::None);
                        program.instruct(Operation::And, program.last(), Operand::Literal(255));
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
