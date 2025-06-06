pub mod intermediate;
pub mod program;

use crate::{
    ast::{
        Assignment, BinaryOperator, Data, Expression, Loop, Primitive, Statement, UnaryOperator,
        Value, Variable,
    },
    semantic::SemanticError,
    types::ast::Initializer,
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
                token: Some(self.identifier.0),
            });
        }

        let values = match self.value {
            Initializer::Expression(value) => vec![value],
            Initializer::List(values) => values,
        };

        for (index, expression) in values.into_iter().enumerate() {
            expression.translate(program)?;
            let identifier = program.infer_name(&self.identifier.0)?;
            let value = program.cast(program.last(), program.type_of(&identifier));
            program.instruct(
                if program.is_global(&self.identifier.0)? {
                    Operation::Stg
                } else {
                    Operation::Str
                },
                Operand::Identifier(
                    program.infer_name(self.identifier.0)?,
                    index + self.identifier.1,
                ),
                value,
            );
        }
        Ok(())
    }
}

impl<'a> Translatable<'a> for Variable<'a> {
    fn translate(self, program: &mut Program<'a>) -> Result<(), SemanticError<'a>> {
        if program.toplevel() {
            match self.assignment {
                Some(Assignment {
                    identifier: (name, _),
                    value: Initializer::Expression(Expression::Value(Value::Data(data))),
                }) => program.define_variable(name, self.datatype, vec![data])?,
                Some(Assignment {
                    identifier: (name, _),
                    value: Initializer::List(values),
                }) => {
                    let data: Vec<_> = (&values)
                        .iter()
                        .filter_map(|value| match value {
                            Expression::Value(Value::Data(data)) => Some(*data),
                            _ => None,
                        })
                        .collect();

                    if data.len() != values.len() {
                        return Err(SemanticError {
                            message: format!(
                                "Top-level variable's '{}' initialization list cannot contain expressions!",
                                self.name
                            ),
                            token: Some(self.name),
                        });
                    }

                    program.define_variable(name, self.datatype, data)?
                }
                Some(_) | None => {
                    return Err(SemanticError {
                        message: format!(
                            "Top-level variable '{}' must be initialized with a constant value!",
                            self.name
                        ),
                        token: Some(self.name),
                    })
                }
            }
        } else {
            program.define_variable(&self.name, self.datatype, vec![])?;
            if let Some(assignment) = self.assignment {
                assignment.translate(program)?;
            }
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

        let loop_start = program.generate_label("loop_start");
        let loop_end = program.generate_label("loop_end");

        self.initialization.translate(program)?;
        program.instruct(
            Operation::Lbl,
            Operand::Label(loop_start.clone()),
            Operand::None,
        );

        self.condition.translate(program)?;
        program.instruct(
            Operation::Cmp,
            program.last(),
            Operand::Data(Data::Integer(0)),
        );
        program.instruct(
            Operation::BEq,
            Operand::Label(loop_end.clone()),
            Operand::None,
        );

        self.body.translate(program)?;
        self.increment.translate(program)?;

        program.instruct(Operation::B, Operand::Label(loop_start), Operand::None);
        program.instruct(Operation::Lbl, Operand::Label(loop_end), Operand::None);
        program.pop_scope();

        Ok(())
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
                    Value::Pointer(identifier, index) => {
                        program.instruct(
                            if program.is_global(identifier)? {
                                Operation::Ldg
                            } else {
                                Operation::Ldr
                            },
                            Operand::Temp,
                            Operand::Identifier(program.infer_name(identifier)?, index),
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
