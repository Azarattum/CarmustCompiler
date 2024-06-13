use crate::{
    ast::Data,
    error::assembly::AssemblyError,
    intermediate::{Operand, Operation},
    program::Program,
};
use std::collections::HashMap;

pub trait Assemblable {
    fn assemble(self) -> Result<String, AssemblyError>;
}

impl Assemblable for Program<'_> {
    fn assemble(self) -> Result<String, AssemblyError> {
        let globals = globals(&self)?;
        let main = main(&self)?;
        Ok(format!(
            ".global main\n{}{}main:\n  {}",
            globals,
            if globals.is_empty() { "" } else { "\n" },
            main
        ))
    }
}

fn globals(program: &Program) -> Result<String, AssemblyError> {
    program
        .globals
        .iter()
        .map(|(name, (datatype, value))| {
            let data = match *value {
                Data::Integer(x) => {
                    if datatype.floating() {
                        (x as f32).to_bits() as i64
                    } else {
                        x as i64
                    }
                }
                Data::Float(x) => {
                    if datatype.floating() {
                        (x as f32).to_bits() as i64
                    } else {
                        x as i64
                    }
                }
            };
            let size = match datatype.size() {
                Some(8) => "xword",
                Some(4) => "word",
                Some(2) => "hword",
                Some(1) => "byte",
                _ => {
                    return Err(AssemblyError {
                        message: format!("Unsupported datatype: {datatype:?}"),
                    })
                }
            };
            Ok(format!("{name}:\n  .{size} {data}"))
        })
        .collect()
}

fn main<'a>(program: &'a Program) -> Result<String, AssemblyError> {
    let mut instructions = vec![format!("sub sp, sp, {}", program.stack_size())];
    let mut addresses: HashMap<usize, usize> = HashMap::new();
    let mut stack: HashMap<&'a str, usize> = HashMap::new();
    let mut registers = [0; 29];

    let mut allocate = |addresses: &mut HashMap<usize, usize>,
                        address: usize,
                        temp: bool,
                        long: bool|
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

        if long {
            Ok(format!("x{index}"))
        } else {
            Ok(format!("w{index}"))
        }
    };

    let mut lookup = |identifier: &'a str| -> Result<String, AssemblyError> {
        match stack.get(identifier) {
            Some(&pointer) => Ok(format!("[sp, {pointer}]")),
            None => {
                match program.globals.get(identifier) {
                    Some(_) => Ok(format!("{identifier}@GOTPAGE")),
                    _ => {
                        let all: usize = *stack.values().max().unwrap_or(&program.stack_size());
                        let pointer = all - 4; // TODO: use datatype size
                        stack.insert(identifier, pointer);
                        Ok(format!("[sp, {pointer}]"))
                    }
                }
            }
        }
    };

    let mut process_operand = |operand: &'a Operand,
                               address: usize,
                               temp: bool,
                               long: bool|
     -> Result<String, AssemblyError> {
        Ok(match operand {
            Operand::Asm(x) => x.to_string(),
            Operand::Data(x) => x.to_string(),
            Operand::None => "".to_owned(),
            Operand::Temp => allocate(&mut addresses, address, temp, long)?,
            Operand::Identifier(x) => lookup(x)?,
            Operand::Address(x) => format!(
                "w{}",
                addresses.get(x).ok_or(AssemblyError {
                    message: format!("Operation at {x} does not have a result register!")
                })?
            ),
        })
    };

    for (address, instruction) in program.instructions.iter().enumerate() {
        // TODO: check type for the right register
        let lhs = process_operand(&instruction.operand1, address, false, false)?;
        let rhs = process_operand(&instruction.operand2, address, false, false)?;

        match instruction.operation {
            Operation::Mov => {
                instructions.push(format!("mov {lhs}, {rhs}"));
            }
            Operation::Add => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("add {result}, {lhs}, {rhs}"));
            }
            Operation::Sub => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("sub {result}, {lhs}, {rhs}"));
            }
            Operation::Mul => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("mul {result}, {lhs}, {rhs}"));
            }
            Operation::SDiv => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("sdiv {result}, {lhs}, {rhs}"));
            }
            Operation::And => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("and {result}, {lhs}, {rhs}"));
            }
            Operation::Orr => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("orr {result}, {lhs}, {rhs}"));
            }
            Operation::Eor => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("eor {result}, {lhs}, {rhs}"));
            }
            Operation::Asr => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("asr {result}, {lhs}, {rhs}"))
            }
            Operation::Lsl => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("lsl {result}, {lhs}, {rhs}"))
            }
            Operation::Cmp => {
                instructions.push(format!("cmp {lhs}, {rhs}"));
            }
            Operation::CSet => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("cset {result}, {lhs}"));
            }
            Operation::Str => {
                instructions.push(format!("str {rhs}, {lhs}"));
            }
            Operation::Ldr => {
                instructions.push(format!("ldr {lhs}, {rhs}"));
            }
            Operation::Ldg => {
                let temp = process_operand(&Operand::Temp, 0, true, true)?;
                instructions.push(format!("adrp {temp}, {rhs}"));
                instructions.push(format!("ldr {temp}, [{temp}, {rhs}OFF]"));
                instructions.push(format!("ldr {lhs}, [{temp}]"));
            }
            Operation::Neg => {
                let result = process_operand(&Operand::Temp, address, false, false)?;
                instructions.push(format!("neg {result}, {lhs}"));
            }
            Operation::Ret => {
                if lhs != "w0" {
                    instructions.push(format!("mov w0, {lhs}"));
                }
                instructions.push(format!("add sp, sp, {}", program.stack_size()));
                instructions.push(format!("ret"));
            }
        }
    }

    return Ok(instructions.join("\n  "));
}
