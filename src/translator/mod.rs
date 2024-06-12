mod intermediate;
pub mod program;

use crate::ast::{
    Assignment, BinaryOperator, Expression, Loop, Statement, UnaryOperator, Value, Variable,
};
use intermediate::{Operand, Operation};
use program::Program;

pub trait Translatable<'a> {
    fn translate(self, program: &mut Program<'a>) -> ();
}

impl<'a> Translatable<'a> for Statement<'a> {
    fn translate(self, program: &mut Program<'a>) {
        match self {
            Self::Type(typedef) => program.define_type(&typedef.name, typedef.datatype),
            Self::Assignment(assignment) => assignment.translate(program),
            Self::Variable(variable) => variable.translate(program),
            Self::Loop(repetition) => repetition.translate(program),
            Self::Return(value) => {
                value.translate(program);
                program.instruct(Operation::Ret, program.last(), Operand::None);
            }
            Self::Function(function) => {
                program.toplevel = false;
                function.body.translate(program);
            }
        }
    }
}

impl<'a> Translatable<'a> for Assignment<'a> {
    fn translate(self, program: &mut Program<'a>) {
        self.value.translate(program);
        program.instruct(
            Operation::Str,
            Operand::Identifier(self.name),
            program.last(),
        )
    }
}

impl<'a> Translatable<'a> for Variable<'a> {
    fn translate(self, program: &mut Program<'a>) -> () {
        program.define_variable(&self.name, self.datatype);
        match self.assignment {
            Some(assignment) => assignment.translate(program),
            None => (),
        }
    }
}

impl<'a> Translatable<'a> for Loop<'a> {
    fn translate(self, program: &mut Program<'a>) -> () {
        todo!()
    }
}

impl<'a> Translatable<'a> for Expression<'a> {
    fn translate(self, program: &mut Program<'a>) {
        match self {
            Self::Value(value) => {
                let operand = match value {
                    Value::Integer(literal) => Operand::Literal(literal),
                    Value::Identifier(identifier) => Operand::Identifier(identifier),
                    _ => todo!(),
                };
                program.instruct(Operation::Mov, Operand::Temp, operand);
            }
            Self::Unary { op, lhs } => {
                lhs.translate(program);
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
                lhs.translate(program);
                let operand1 = program.last();
                rhs.translate(program);
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
        }
    }
}

impl<'a> Translatable<'a> for Vec<Statement<'a>> {
    fn translate(self, program: &mut Program<'a>) {
        self.into_iter().for_each(|x| x.translate(program));
    }
}
