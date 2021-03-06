use std::{fs, io};
use wasm_builder::*;

fn main() -> io::Result<()> {
    let mut module = module::Module::new();

    let add = sections::Function {
        locals: vec![],
        body: instr::Expr(vec![
            instr::Instruction::LocalGet(0),
            instr::Instruction::LocalGet(1),
            instr::Instruction::Add(types::ValType::F32),
        ]),
    };

    module.types.push(types::FunctionType {
        parameter_types: vec![types::ValType::F32, types::ValType::F32],
        return_types: vec![types::ValType::F32],
    });
    module.functions.push(0);
    module.code.push(add);
    module.exports.push(sections::Export {
        name: String::from("add"),
        desc: sections::Desc::Function(0),
    });

    let mut file = fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open("./add.wasm")?;
    module.encode(&mut file)?;

    Ok(())
}
