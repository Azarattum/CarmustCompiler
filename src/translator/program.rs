use super::intermediate::{Instruction, Operand, Operation};
use crate::{
    ast::{Data, DataType, Primitive},
    semantic::SemanticError,
};
use std::{collections::HashMap, fmt::Debug};

pub struct Program<'a> {
    pub globals: HashMap<String, (DataType<'a>, Data)>,
    pub locals: HashMap<String, DataType<'a>>,
    pub instructions: Vec<Instruction>,

    types: HashMap<&'a str, DataType<'a>>,
    scope: usize,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Program {
            scope: 0,
            types: HashMap::new(),
            instructions: Vec::new(),
            locals: HashMap::new(),
            globals: HashMap::new(),
        }
    }

    pub fn toplevel(&self) -> bool {
        self.scope == 0
    }

    pub fn push_scope(&mut self) -> () {
        self.scope += 1;
    }

    pub fn pop_scope(&mut self) -> () {
        self.scope -= 1;
    }

    fn infer_scope(&self, name: &'a str) -> Result<usize, SemanticError<'a>> {
        (1..=self.scope)
            .rev()
            .find(|i| self.locals.contains_key(format!("{name}_{i}").as_str()))
            .or_else(|| {
                if self.globals.contains_key(format!("{name}_0").as_str()) {
                    Some(0)
                } else {
                    None
                }
            })
            .ok_or(SemanticError {
                message: format!("'{}' is not defined!", name),
                token: Some(name),
            })
    }

    pub fn infer_name(&self, name: &'a str) -> Result<String, SemanticError<'a>> {
        let scope = self.infer_scope(name)?;
        Ok(format!("{name}_{}", scope))
    }

    pub fn is_global(&self, name: &'a str) -> Result<bool, SemanticError<'a>> {
        Ok(self.infer_scope(name)? == 0)
    }

    fn is_defined_here(&mut self, name: &'a str) -> bool {
        self.locals.contains_key(self.local_name(name).as_str())
    }

    fn local_name(&self, name: &'a str) -> String {
        format!("{name}_{}", self.scope)
    }

    fn resolve_type(&self, datatype: DataType<'a>) -> Result<DataType<'a>, SemanticError<'a>> {
        match datatype {
            DataType::Primitive(Primitive::Custom(alias)) => match self.types.get(alias) {
                Some(datatype) => self.resolve_type(*datatype),
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

        let datatype = self.resolve_type(datatype)?;
        if self.toplevel() {
            match value {
                None => {
                    return Err(SemanticError {
                        message: format!("Top-level variable '{}' must be initialized!", name),
                        token: Some(name),
                    })
                }
                Some(value) => {
                    self.globals
                        .insert(self.local_name(name), (datatype, value));
                }
            }
        } else {
            self.locals.insert(self.local_name(name), datatype);
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

    pub fn last(&self) -> Operand {
        Operand::Address(self.instructions.len() - 1)
    }

    pub fn instruct(&mut self, operation: Operation, operand1: Operand, operand2: Operand) {
        self.instructions.push(Instruction {
            operation,
            operand1,
            operand2,
        });
    }

    pub fn stack_size(&self) -> usize {
        let size = self
            .locals
            .iter()
            .map(|(_, &datatype)| datatype.size())
            .sum::<Option<usize>>()
            .unwrap_or(0);

        size + (16 - size % 16)
    }
}

impl<'a> Debug for Program<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "globals:\n")?;
        for (key, (_, value)) in self.globals.iter() {
            match value {
                Data::Float(x) => write!(f, "  {key} = {x:e}\n")?,
                x => write!(f, "  {key} = {x}\n")?,
            }
        }
        write!(f, "\nmain:\n")?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            write!(f, "{:3}) {:?}\n", i, instruction)?;
        }
        Ok(())
    }
}
