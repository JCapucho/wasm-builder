use std::io::{self, Write};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ValType {
    Int,
    Long,
    Float,
    Double,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Limits {
    pub fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        match self.max {
            Some(max) => {
                writer.write(&[0x01])?;
                encode_u32(writer, self.min)?;
                encode_u32(writer, max)?;
            }
            None => {
                writer.write(&[0x00])?;
                encode_u32(writer, self.min)?;
            }
        };

        Ok(())
    }
}

pub(crate) fn encode_u32(writer: &mut impl Write, val: u32) -> io::Result<usize> {
    let bytes = leb128::write::unsigned(writer, val as u64)?;
    assert!(bytes <= (32f32 / 7.0).ceil() as usize);
    Ok(bytes)
}

pub(crate) fn encode_i32(writer: &mut impl Write, val: i32) -> io::Result<usize> {
    let bytes = leb128::write::signed(writer, val as i64)?;
    assert!(bytes <= (32f32 / 7.0).ceil() as usize);
    Ok(bytes)
}

pub(crate) fn encode_i64(writer: &mut impl Write, val: i64) -> io::Result<usize> {
    let bytes = leb128::write::signed(writer, val)?;
    assert!(bytes <= (64f32 / 7.0).ceil() as usize);
    Ok(bytes)
}

pub(crate) fn encode_f32(writer: &mut impl Write, val: f32) -> io::Result<usize> {
    writer.write(&val.to_le_bytes())
}

pub(crate) fn encode_f64(writer: &mut impl Write, val: f64) -> io::Result<usize> {
    writer.write(&val.to_le_bytes())
}

pub(crate) fn encode_vec(writer: &mut impl Write, bytes: &[u8], size: u32) -> io::Result<usize> {
    let mut length = encode_u32(writer, size)?;
    length += writer.write(bytes)?;
    Ok(length)
}

pub(crate) fn encode_name(writer: &mut impl Write, val: &str) -> io::Result<usize> {
    encode_vec(writer, val.as_bytes(), val.chars().count() as u32)
}

pub(crate) fn encode_val_type(writer: &mut impl Write, ty: ValType) -> io::Result<usize> {
    match ty {
        ValType::Int => writer.write(&[0x7F]),
        ValType::Long => writer.write(&[0x7E]),
        ValType::Float => writer.write(&[0x7D]),
        ValType::Double => writer.write(&[0x7C]),
    }
}

pub(crate) fn encode_result_type(writer: &mut impl Write, types: &[ValType]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(types.len() + 1);

    for ty in types {
        encode_val_type(&mut buf, *ty)?;
    }

    encode_vec(writer, &buf, types.len() as u32)?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub parameter_types: Vec<ValType>,
    pub return_types: Vec<ValType>,
}

impl FunctionType {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write(&[0x60])?;

        encode_result_type(writer, &self.parameter_types)?;
        encode_result_type(writer, &self.return_types)?;

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct MemoryType {
    pub lim: Limits,
}

impl MemoryType {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        self.lim.encode(writer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TableType {
    pub lim: Limits,
}

impl TableType {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write(&[0x70])?;
        self.lim.encode(writer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlobalType {
    pub ty: ValType,
    pub mutable: bool,
}

impl GlobalType {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<()> {
        encode_val_type(writer, self.ty)?;
        match self.mutable {
            true => writer.write(&[0x00]),
            false => writer.write(&[0x01]),
        }?;
        Ok(())
    }
}
