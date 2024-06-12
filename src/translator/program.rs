use super::intermediate::{Instruction, Operand, Operation};
use crate::{
    ast::{DataType, Primitive},
    semantic::SemanticError,
};
use std::{collections::HashMap, fmt::Debug};

pub struct Program<'a> {
    scope: Vec<HashMap<&'a str, (DataType<'a>, Option<i64>)>>, // TODO: other value types
    types: HashMap<&'a str, DataType<'a>>,
    instructions: Vec<Instruction<'a>>,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Program {
            types: HashMap::new(),
            instructions: Vec::new(),
            scope: vec![HashMap::new()],
        }
    }

    pub fn toplevel(&self) -> bool {
        self.scope.len() == 1
    }

    pub fn globals(&self) -> &HashMap<&'a str, (DataType<'a>, Option<i64>)> {
        &self.scope[0]
    }

    pub fn current_scope(&mut self) -> &mut HashMap<&'a str, (DataType<'a>, Option<i64>)> {
        let index = self.scope.len() - 1;
        unsafe { self.scope.get_unchecked_mut(index) }
    }

    pub fn push_scope(&mut self) -> () {
        self.scope.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) -> () {
        self.scope.pop();
    }

    pub fn is_defined(&self, name: &'a str) -> bool {
        self.scope.iter().any(|scope| scope.contains_key(name))
    }

    pub fn is_defined_here(&mut self, name: &'a str) -> bool {
        self.current_scope().contains_key(name)
    }

    pub fn resolve(&self, datatype: DataType<'a>) -> Result<DataType<'a>, SemanticError<'a>> {
        match datatype {
            DataType::Primitive(Primitive::Custom(alias)) => match self.types.get(alias) {
                Some(datatype) => self.resolve(*datatype),
                None => Err(SemanticError {
                    message: format!("Type '{}' is not defined!", alias),
                    token: Some(alias),
                }),
            },
            _ => Ok(datatype),
        }
    }

    pub fn define_variable(
        &mut self,
        name: &'a str,
        datatype: DataType<'a>,
        value: Option<i64>,
    ) -> Result<(), SemanticError<'a>> {
        if self.is_defined_here(name) {
            return Err(SemanticError {
                message: format!("Variable '{}' is already defined in this scope!", name),
                token: Some(name),
            });
        }

        let datatype = self.resolve(datatype)?;
        if self.toplevel() {
            if value.is_none() {
                return Err(SemanticError {
                    message: format!("Top-level variable '{}' must be initialized!", name),
                    token: Some(name),
                });
            }

            self.current_scope().insert(name, (datatype, value));
        } else {
            self.current_scope().insert(name, (datatype, None));
        }

        Ok(())
    }

    pub fn define_type(
        &mut self,
        name: &'a str,
        datatype: DataType<'a>,
    ) -> Result<(), SemanticError<'a>> {
        self.types.insert(name, datatype);
        Ok(())
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
        write!(f, "globals:\n")?;
        for (key, (_, value)) in self.globals().iter() {
            write!(f, "  {} = {}\n", key, value.unwrap())?;
        }
        write!(f, "\nmain:\n")?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            write!(f, "{:3}) {:?}\n", i, instruction)?;
        }
        Ok(())
    }
}
