use crate::ast::{DataType, Primitive};
use std::{collections::HashMap, fmt::Debug};

use super::intermediate::{Instruction, Operand, Operation};

pub struct Program<'a> {
    pub toplevel: bool,
    scope: HashMap<&'a str, DataType<'a>>,
    types: HashMap<&'a str, DataType<'a>>,
    instructions: Vec<Instruction<'a>>,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Program {
            toplevel: true,
            scope: HashMap::new(),
            types: HashMap::new(),
            instructions: Vec::new(),
        }
    }

    pub fn resolve(&self, datatype: DataType<'a>) -> DataType<'a> {
        match datatype {
            DataType::Primitive(Primitive::Custom(alias)) => {
                self.resolve(*(self.types.get(alias).unwrap()))
            }
            _ => datatype,
        }
    }

    pub fn define_variable(&mut self, name: &'a str, datatype: DataType<'a>) {
        // TODO: check for existence
        self.scope.insert(name, self.resolve(datatype));
    }

    pub fn define_type(&mut self, name: &'a str, datatype: DataType<'a>) {
        // TODO: check for existence
        self.types.insert(name, datatype);
    }

    pub fn last(&self) -> Operand<'a> {
        Operand::Address(self.instructions.len() - 1)
    }

    pub fn instruct(&mut self, operation: Operation, operand1: Operand<'a>, operand2: Operand<'a>) {
        self.instructions.push(Instruction {
            operation,
            operand1,
            operand2,
        });
    }
}

impl<'a> Debug for Program<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, instruction) in self.instructions.iter().enumerate() {
            write!(f, "{:3}) {:?}\n", i, instruction)?;
        }
        Ok(())
    }
}
