use super::intermediate::{Instruction, Operand, Operation};
use crate::{
    ast::{Data, DataType, Primitive},
    semantic::SemanticError,
};
use std::{collections::HashMap, fmt::Debug};

pub struct Program<'a> {
    scope: Vec<HashMap<&'a str, (DataType<'a>, Option<Data>)>>,
    types: HashMap<&'a str, DataType<'a>>,
    instructions: Vec<Instruction<'a>>,
    stack: usize,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Program {
            stack: 0,
            types: HashMap::new(),
            instructions: Vec::new(),
            scope: vec![HashMap::new()],
        }
    }

    pub fn toplevel(&self) -> bool {
        self.scope.len() == 1
    }

    pub fn globals(&self) -> &HashMap<&'a str, (DataType<'a>, Option<Data>)> {
        &self.scope[0]
    }

    pub fn current_scope(&mut self) -> &mut HashMap<&'a str, (DataType<'a>, Option<Data>)> {
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

    pub fn is_global(&self, name: &'a str) -> bool {
        self.globals().contains_key(name)
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
        value: Option<Data>,
    ) -> Result<(), SemanticError<'a>> {
        if self.is_defined_here(name) {
            return Err(SemanticError {
                message: format!("Variable '{}' is already defined in this scope!", name),
                token: Some(name),
            });
        }

        let datatype = self.resolve(datatype)?;
        self.stack += datatype.size().unwrap();
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

    pub fn instructions(&self) -> &Vec<Instruction<'a>> {
        &self.instructions
    }

    pub fn stack_size(&self) -> usize {
        self.stack + (16 - self.stack % 16)
    }
}

impl<'a> Debug for Program<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "globals:\n")?;
        for (key, (_, value)) in self.globals().iter() {
            match value {
                Some(Data::Integer(x)) => write!(f, "  {} = {}\n", key, x)?,
                Some(Data::Float(x)) => write!(f, "  {} = {:e}\n", key, x)?,
                _ => (),
            }
        }
        write!(f, "\nmain:\n")?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            write!(f, "{:3}) {:?}\n", i, instruction)?;
        }
        Ok(())
    }
}
