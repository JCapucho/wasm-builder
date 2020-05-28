use crate::{sections, types};
use std::io::{self, Write};

// The WASM magic byte sequence (\0asm) needed in every module
const MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];
const VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00]; // Version 1

/// Represents a wasm binary module
///
/// The binary encoding of a module is organized into sections.

/// Most sections correspond to one component of a module record,
/// except that function definitions are split into two sections,
/// separating their type declarations in the function section from
/// their bodies in the code section.
#[derive(Debug, Clone)]
pub struct Module<'a> {
    /// types section
    pub types: Vec<types::FunctionType>,
    /// imports section
    pub imports: Vec<sections::Import>,
    /// functions section
    pub functions: Vec<sections::TypeIdx>,
    /// tables section
    pub tables: Vec<types::TableType>,
    /// memory section
    pub memory: Vec<types::MemoryType>,
    /// globals section
    pub globals: Vec<sections::Global>,
    /// exports section
    pub exports: Vec<sections::Export>,
    /// start section
    pub start: Option<sections::FuncIdx>,
    /// elements section
    pub elements: Vec<sections::Element>,
    /// code section
    pub code: Vec<sections::Function>,
    /// data section
    pub data: Vec<sections::Data<'a>>,
}

impl<'a> Module<'a> {
    /// Creates a empty Module
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

    /// Writes the binary wasm to a type implementing Write
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
