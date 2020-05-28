use crate::{instr::Expr, types};
use std::io::{self, Write};

pub type LabelIdx = u32;
pub type FuncIdx = u32;
pub type TypeIdx = u32;
pub type LocalIdx = u32;
pub type GlobalIdx = u32;
pub type MemoryIdx = u32;
pub type TableIdx = u32;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum Section {
    Custom = 0,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
}

/// Describes an import or export
#[derive(Debug, Copy, Clone)]
pub enum Desc {
    /// A function index
    Function(TypeIdx),
    /// A table
    Table(types::TableType),
    /// A memory
    Memory(types::MemoryType),
    /// A global
    Global(types::GlobalType),
}

impl Desc {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        match self {
            Desc::Function(func) => {
                // Function identifier: 0x00
                writer.write(&[0x00])?;
                types::encode_u32(writer, *func)?;
            }
            Desc::Table(table) => {
                // Table identifier: 0x01
                writer.write(&[0x01])?;
                table.encode(writer)?;
            }
            Desc::Memory(mem) => {
                // Memory identifier: 0x02
                writer.write(&[0x02])?;
                mem.encode(writer)?;
            }
            Desc::Global(global) => {
                // Global identifier: 0x03
                writer.write(&[0x03])?;
                global.encode(writer)?;
            }
        }

        Ok(())
    }
}

/// The import component defines a import that is need for
/// [instantiation](https://webassembly.github.io/spec/core/exec/modules.html#exec-instantiation)
#[derive(Debug, Clone)]
pub struct Import {
    /// The module name
    pub module: String,
    /// The import name
    pub name: String,
    /// The import itself
    pub desc: Desc,
}

impl Import {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        types::encode_name(writer, &self.module)?;
        types::encode_name(writer, &self.name)?;
        self.desc.encode(writer)
    }
}

/// The global component defines a global variable
#[derive(Debug, Clone)]
pub struct Global {
    /// The type of the global
    pub ty: types::GlobalType,
    /// The init expression of the global
    pub init: Expr,
}

impl Global {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        self.ty.encode(writer)?;
        self.init.encode(writer)?;
        Ok(())
    }
}

/// The export component defines a export that becomes accessible
/// to the host environment once the module has been instantiated
#[derive(Debug, Clone)]
pub struct Export {
    /// The name of the export
    pub name: String,
    /// The export itself
    pub desc: Desc,
}

impl Export {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        types::encode_name(writer, &self.name)?;
        self.desc.encode(writer)
    }
}

/// The Element component provides a way to initialize a subrange of a table
#[derive(Debug, Clone)]
pub struct Element {
    /// The table being initialized
    pub table: TableIdx,
    /// The expression that gives the offset into the table
    pub offset: Expr,
    /// The data to fill the subrange
    pub init: Vec<FuncIdx>,
}

impl Element {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        types::encode_u32(writer, self.table)?;
        self.offset.encode(writer)?;

        let mut buf = Vec::with_capacity(std::mem::size_of_val(&self.init));

        for idx in self.init.iter() {
            types::encode_u32(&mut buf, *idx)?;
        }

        types::encode_vec(writer, &buf, self.init.len() as u32)?;
        Ok(())
    }
}

/// Defines a mutable local variable
///
/// Locals are referenced by their index
///
/// The index of the first local is the smallest index not referencing a parameter
#[derive(Debug, Clone)]
pub struct Local {
    /// The local index
    pub n: u32,
    /// The type of the local
    pub ty: types::ValType,
}

impl Local {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        types::encode_u32(writer, self.n)?;
        types::encode_val_type(writer, self.ty)?;
        Ok(())
    }
}

/// Defines a function component
#[derive(Debug, Clone)]
pub struct Function {
    /// The functions locals
    pub locals: Vec<Local>,
    /// The function body
    pub body: Expr,
}

impl Function {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<usize> {
        let mut buf = Vec::with_capacity(std::mem::size_of_val(&self.locals));

        for local in self.locals.iter() {
            local.encode(&mut buf)?;
        }

        let mut length = types::encode_vec(writer, &buf, self.locals.len() as u32)?;
        length += self.body.encode(writer)?;
        Ok(length)
    }
}

/// The data component defines a vector of data to initialize a subrange of a memory
#[derive(Debug, Clone)]
pub struct Data<'a> {
    /// The memory being initialized
    pub mem: MemoryIdx,
    /// The offset into the memory
    pub offset: Expr,
    /// The data to initialize the subrange with
    pub init: &'a [u8],
}

impl<'a> Data<'a> {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        types::encode_u32(writer, self.mem)?;
        self.offset.encode(writer)?;
        types::encode_vec(writer, self.init, self.init.len() as u32)?;
        Ok(())
    }
}

fn encode_section_header(writer: &mut impl Write, id: Section, size: u32) -> io::Result<()> {
    writer.write(&[id as u8])?;

    types::encode_u32(writer, size)?;

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn encode_custom_section(
    writer: &mut impl Write,
    name: &str,
    data: &[u8],
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(data.len() + name.len());

    types::encode_name(&mut buf, name)?;
    buf.write(data)?;

    encode_section_header(writer, Section::Custom, std::mem::size_of_val(&buf) as u32)?;
    writer.write(&buf)?;

    Ok(())
}

pub(crate) fn encode_type_section(
    writer: &mut impl Write,
    section: &[types::FunctionType],
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Type, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_import_section(writer: &mut impl Write, section: &[Import]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Import, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_function_section(
    writer: &mut impl Write,
    section: &[TypeIdx],
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        types::encode_u32(&mut buf, *ty)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Function, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_table_section(
    writer: &mut impl Write,
    section: &[types::TableType],
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Table, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_memory_section(
    writer: &mut impl Write,
    section: &[types::MemoryType],
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Memory, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_global_section(writer: &mut impl Write, section: &[Global]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Global, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_export_section(writer: &mut impl Write, section: &[Export]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Export, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_start_section(writer: &mut impl Write, start: FuncIdx) -> io::Result<()> {
    let mut buf = Vec::with_capacity(4);

    let size = types::encode_u32(&mut buf, start)?;

    encode_section_header(writer, Section::Start, size as u32)?;
    writer.write(&buf)?;

    Ok(())
}

pub(crate) fn encode_element_section(
    writer: &mut impl Write,
    section: &[Element],
) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for ty in section {
        ty.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Element, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

fn encode_code(writer: &mut impl Write, func: &Function) -> io::Result<()> {
    let mut buf = Vec::new();
    let size = func.encode(&mut buf)?;
    types::encode_u32(writer, size as u32)?;
    writer.write(&buf)?;

    Ok(())
}

pub(crate) fn encode_code_section(writer: &mut impl Write, section: &[Function]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for func in section {
        encode_code(&mut buf, func)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Code, size as u32)?;
    writer.write(&data)?;

    Ok(())
}

pub(crate) fn encode_data_section(writer: &mut impl Write, section: &[Data]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(std::mem::size_of_val(&section));

    for data in section {
        data.encode(&mut buf)?;
    }

    let mut data = Vec::with_capacity(buf.len() + 4);
    let size = types::encode_vec(&mut data, &buf, section.len() as u32)?;
    encode_section_header(writer, Section::Data, size as u32)?;
    writer.write(&data)?;

    Ok(())
}
