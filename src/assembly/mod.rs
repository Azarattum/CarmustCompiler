mod arm;
mod operation;

use crate::{
    ast::Primitive, error::assembly::AssemblyError, intermediate::Operand, program::Program,
};
use arm::*;
use operation::AssemblablePart;
use std::collections::HashMap;

pub trait Assemblable {
    fn assemble(self) -> Result<String, AssemblyError>;
}

impl Assemblable for Program<'_> {
    fn assemble(self) -> Result<String, AssemblyError> {
        let globals = globals(&self)?;
        let main = main(&self)?;

        Ok(format!(
            "{}.global main\nmain:\n{}",
            if globals.is_empty() {
                "".to_owned()
            } else {
                format!(".section __DATA,__data\n{globals}\n\n.section __TEXT,__text\n")
            },
            main
        ))
    }
}

fn globals(program: &Program) -> Result<String, AssemblyError> {
    program
        .globals
        .iter()
        .map(|(name, (datatype, values))| {
            let definitions = values
                .into_iter()
                .map(|value| {
                    let data = if datatype.0.floating() {
                        f32::from(value).to_bits() as i64
                    } else {
                        i64::from(value)
                    };
                    let size = match datatype.0.size() {
                        8 => "xword",
                        4 => "word",
                        2 => "hword",
                        1 => "byte",
                        _ => {
                            return Err(AssemblyError {
                                message: format!("Unsupported datatype: {datatype:?}"),
                            })
                        }
                    };
                    Ok(format!("  .{size} {data}"))
                })
                .collect::<Result<Vec<_>, AssemblyError>>();

            Ok(format!("{name}:\n{}", definitions?.join("\n")))
        })
        .intersperse(Ok("\n".to_owned()))
        .collect()
}

fn main<'a>(program: &'a Program) -> Result<String, AssemblyError> {
    let mut instructions = vec![format!("sub sp, sp, {}", program.stack_size())];
    let mut addresses: HashMap<usize, usize> = HashMap::new();
    let mut stack: HashMap<String, usize> = HashMap::new();
    let mut registers = [0; 29];

    let mut allocate = |addresses: &mut HashMap<usize, usize>,
                        address: usize,
                        temp: bool|
     -> Result<String, AssemblyError> {
        let index = registers
            .iter()
            .position(|&x| x <= address)
            .ok_or(AssemblyError {
                message: format!("Compilation impossible! Ran out of registers!"),
            })?;

        if !temp {
            registers[index] = program
                .instructions
                .iter()
                .rposition(|x| {
                    x.operand1 == Operand::Address(address)
                        || x.operand2 == Operand::Address(address)
                })
                .unwrap_or(address);
            addresses.insert(address, index);
        }

        Ok(index.to_string())
    };

    let mut lookup = |identifier: &str, index: usize| -> Result<String, AssemblyError> {
        let offset = index * program.type_of(identifier).unwrap().size();
        match stack.get(identifier) {
            Some(&pointer) => Ok(format!("[sp, {}]", pointer + offset)),
            None => match program.globals.get(identifier) {
                Some(_) => Ok(format!("{identifier}@{offset}")),
                _ => {
                    let all: usize = *stack.values().min().unwrap_or(&program.stack_size());
                    let pointer = all - program.locals.get(identifier).unwrap().size();
                    stack.insert(identifier.to_owned(), pointer);
                    Ok(format!("[sp, {}]", pointer + offset))
                }
            },
        }
    };

    let mut process_operand = |operand: &Operand,
                               address: usize,
                               datatype: Option<Primitive>,
                               temp: bool|
     -> Result<String, AssemblyError> {
        Ok(match operand {
            Operand::Identifier(x, offset) => lookup(x, *offset)?,
            Operand::Label(label) => label.clone(),
            Operand::Data(x) => x.represent(),
            Operand::Asm(x) => x.to_string(),
            Operand::None => "".to_owned(),
            Operand::Temp => format!(
                "{}{}",
                as_register(datatype.ok_or(AssemblyError {
                    message: format!("Unable to infer a type for register at {address}!"),
                })?),
                allocate(&mut addresses, address, temp)?
            ),
            Operand::Address(x) => format!(
                "{}{}",
                as_register(datatype.ok_or(AssemblyError {
                    message: format!("Operation at {x} does not have a type!"),
                })?),
                addresses.get(x).ok_or(AssemblyError {
                    message: format!("Operation at {x} does not have a result register!")
                })?
            ),
        })
    };

    for (address, cmd) in program.instructions.iter().enumerate() {
        let result_type = cmd.datatype(program);
        let lhs_type = cmd.operand1.datatype(program).or(result_type);
        let rhs_type = cmd.operand2.datatype(program).or(result_type);

        let lhs = process_operand(&cmd.operand1, address, lhs_type, false)?;
        let rhs = process_operand(&cmd.operand2, address, rhs_type, false)?;

        let allocate = |temp: bool, datatype: Option<Primitive>| {
            let address = if temp { 0 } else { address };
            process_operand(&Operand::Temp, address, datatype, temp)
        };

        instructions.extend(cmd.operation.assemble(allocate, result_type, lhs, rhs)?);
    }

    let index = if let Some(x) = instructions.last()
        && x.starts_with("ret")
    {
        instructions.len() - 1
    } else {
        instructions.len()
    };
    instructions.insert(index, format!("add sp, sp, {}", program.stack_size()));

    let formatted_instructions: Vec<_> = instructions
        .into_iter()
        .map(|instruction| {
            if instruction.ends_with(":") {
                instruction
            } else {
                format!("  {}", instruction)
            }
        })
        .collect();

    return Ok(formatted_instructions.join("\n"));
}

fn as_register(datatype: Primitive) -> &'static str {
    match datatype {
        Primitive::Byte | Primitive::Short | Primitive::Int => "w",
        Primitive::Float => "s",
        Primitive::Long => "x",
    }
}
