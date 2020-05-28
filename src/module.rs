use crate::{sections, types};
use std::io::{self, Write};

const MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
const VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00]; // Version 1

#[derive(Debug, Clone)]
pub struct Module<'a> {
    pub types: Vec<types::FunctionType>,
    pub imports: Vec<sections::Import>,
    pub functions: Vec<sections::TypeIdx>,
    pub tables: Vec<types::TableType>,
    pub memory: Vec<types::MemoryType>,
    pub globals: Vec<sections::Global>,
    pub exports: Vec<sections::Export>,
    pub start: Option<sections::FuncIdx>,
    pub elements: Vec<sections::Element>,
    pub code: Vec<sections::Function>,
    pub data: Vec<sections::Data<'a>>,
}

impl<'a> Module<'a> {
    pub fn new() -> Self {
        Module {
            types: vec![],
            imports: vec![],
            functions: vec![],
            tables: vec![],
            memory: vec![],
            globals: vec![],
            exports: vec![],
            start: None,
            elements: vec![],
            code: vec![],
            data: vec![],
        }
    }

    pub fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write(&MAGIC)?;
        writer.write(&VERSION)?;
        if self.types.len() != 0 {
            sections::encode_type_section(writer, &self.types)?;
        }
        if self.imports.len() != 0 {
            sections::encode_import_section(writer, &self.imports)?;
        }
        if self.functions.len() != 0 {
            sections::encode_function_section(writer, &self.functions)?;
        }
        if self.tables.len() != 0 {
            sections::encode_table_section(writer, &self.tables)?;
        }
        if self.memory.len() != 0 {
            sections::encode_memory_section(writer, &self.memory)?;
        }
        if self.globals.len() != 0 {
            sections::encode_global_section(writer, &self.globals)?;
        }
        if self.exports.len() != 0 {
            sections::encode_export_section(writer, &self.exports)?;
        }
        if let Some(start) = self.start {
            sections::encode_start_section(writer, start)?;
        }
        if self.elements.len() != 0 {
            sections::encode_element_section(writer, &self.elements)?;
        }
        if self.code.len() != 0 {
            sections::encode_code_section(writer, &self.code)?;
        }
        if self.data.len() != 0 {
            sections::encode_data_section(writer, &self.data)?;
        }

        Ok(())
    }
}
