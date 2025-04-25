use super::intermediate::{Instruction, Operand, Operation};
use crate::{
    ast::{Compound, Data, Datatype, Primitive},
    semantic::SemanticError,
};
use std::{collections::HashMap, fmt::Debug};

pub struct Program<'a> {
    pub globals: HashMap<String, (Compound, Vec<Data>)>,
    pub locals: HashMap<String, Compound>,
    pub instructions: Vec<Instruction>,

    types: HashMap<&'a str, Compound>,
    scope: usize,
    label: usize,
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Program {
            scope: 0,
            label: 0,
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

    fn resolve_type(&self, datatype: Datatype<'a>) -> Result<Compound, SemanticError<'a>> {
        match datatype {
            Datatype::Type(datatype) => Ok(datatype),
            Datatype::Alias(alias) => match self.types.get(alias) {
                Some(datatype) => Ok(*datatype),
                None => Err(SemanticError {
                    message: format!("Type '{}' is not defined!", alias),
                    token: Some(alias),
                }),
            },
        }
    }

    pub fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label);
        self.label += 1;
        label
    }

    pub fn type_of(&self, identifier: &'a str) -> Option<Primitive> {
        self.locals
            .get(identifier)
            .or_else(|| self.globals.get(identifier).and_then(|x| Some(&x.0)))
            .and_then(|x| Some(x.0))
    }

    pub fn define_variable(
        &mut self,
        name: &'a str,
        datatype: Datatype<'a>,
        value: Vec<Data>,
    ) -> Result<(), SemanticError<'a>> {
        if self.is_defined_here(name) {
            return Err(SemanticError {
                message: format!("Variable '{}' is already defined in this scope!", name),
                token: Some(name),
            });
        }

        let datatype = self.resolve_type(datatype)?;
        if self.toplevel() {
            self.globals
                .insert(self.local_name(name), (datatype, value));
        } else {
            self.locals.insert(self.local_name(name), datatype);
        }

        Ok(())
    }

    pub fn define_type(
        &mut self,
        name: &'a str,
        datatype: Datatype<'a>,
    ) -> Result<(), SemanticError<'a>> {
        self.types.insert(name, self.resolve_type(datatype)?);
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
        let size: usize = self
            .locals
            .iter()
            .map(|(_, &datatype)| datatype.size())
            .sum();

        size + (16 - size % 16)
    }

    pub fn cast(&mut self, operand: Operand, to: Option<Primitive>) -> Operand {
        let from = operand.datatype(self);
        let cast = match (from, to) {
            (from, to) if from == to => None,
            (
                Some(Primitive::Byte | Primitive::Short | Primitive::Int | Primitive::Long),
                Some(Primitive::Float),
            ) => Some(Operation::SCvtF),
            (
                Some(Primitive::Float),
                Some(Primitive::Byte | Primitive::Short | Primitive::Int | Primitive::Long),
            ) => Some(Operation::FCvtZS),
            _ => None,
        };

        if let Some(instruction) = cast {
            self.instruct(instruction, operand, Operand::None);
            self.last()
        } else {
            operand
        }
    }
}

impl<'a> Debug for Program<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "globals:\n")?;
        for (key, (_, values)) in self.globals.iter() {
            let representation: Vec<_> = values
                .into_iter()
                .map(|value| match value {
                    Data::Float(x) => format!("{x:e}"),
                    x => format!("{x}"),
                })
                .collect();

            let representation = if representation.len() == 1 {
                &representation[0]
            } else {
                &format!("[{}]", representation.join(", "))
            };

            write!(f, "  {key} = {representation}\n")?
        }
        write!(f, "\nmain:\n")?;
        for (i, instruction) in self.instructions.iter().enumerate() {
            write!(f, "{:3}) {:?}\n", i, instruction)?;
        }
        Ok(())
    }
}
