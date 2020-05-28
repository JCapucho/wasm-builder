use super::sections::*;
use super::types;
use std::io::{self, Write};

#[derive(Debug, Copy, Clone)]
pub enum BlockType {
    Empty,
    Type(types::ValType),
    TypeIdx(u32),
}

impl BlockType {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<usize> {
        match self {
            BlockType::Empty => writer.write(&[0x40]),
            BlockType::Type(ty) => types::encode_val_type(writer, *ty),
            BlockType::TypeIdx(idx) => types::encode_i64(writer, *idx as i64),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryArgument {
    pub alignment: u32,
    pub offset: u32,
}

impl MemoryArgument {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<usize> {
        let mut length = types::encode_u32(writer, self.alignment)?;
        length += types::encode_u32(writer, self.offset)?;
        Ok(length)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MemoryType {
    Int,    // i32
    Long,   // i64
    Float,  // f32
    Double, // f64
}

#[derive(Debug, Copy, Clone)]
pub enum StorageType {
    Byte,  // 8
    Short, // 16
    Int,   // 32
}

#[derive(Debug, Copy, Clone)]
pub enum Literal {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

#[derive(Debug, Copy, Clone)]
pub enum IntegerType {
    Int,
    Long,
}

#[derive(Debug, Copy, Clone)]
pub enum FloatType {
    Float,
    Double,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Unreachable,
    NOP,
    Block {
        ty: BlockType,
        instrs: Vec<Instruction>,
    },
    Loop {
        ty: BlockType,
        instrs: Vec<Instruction>,
    },
    If {
        ty: BlockType,
        accept_instrs: Vec<Instruction>,
        reject_instrs: Option<Vec<Instruction>>,
    },
    Branch(LabelIdx),
    BranchIf(LabelIdx),
    BranchTable {
        labels: Vec<LabelIdx>,
        operand: LabelIdx,
    },
    Return,
    Call(FuncIdx),
    CallIndirect(TypeIdx),
    Drop,
    Select,
    LocalGet(LocalIdx),
    LocalSet(LocalIdx),
    LocalTee(LocalIdx),
    GlobalGet(GlobalIdx),
    GlobalSet(GlobalIdx),
    Load {
        mem: MemoryArgument,
        ty: MemoryType,
        storage: Option<(bool, StorageType)>,
    },
    Store {
        mem: MemoryArgument,
        ty: MemoryType,
        storage: Option<StorageType>,
    },
    MemorySize,
    MemoryGrow,
    Const(Literal),
    EqualZero(IntegerType),
    Equal(MemoryType),
    NotEqual(MemoryType),
    LessThanInt {
        ty: IntegerType,
        signed: bool,
    },
    GreaterThanInt {
        ty: IntegerType,
        signed: bool,
    },
    LessOrEqualInt {
        ty: IntegerType,
        signed: bool,
    },
    GreaterOrEqualInt {
        ty: IntegerType,
        signed: bool,
    },
    LessThanFloat(FloatType),
    GreaterThanFloat(FloatType),
    LessOrEqualFloat(FloatType),
    GreaterOrEqualFloat(FloatType),
    CountLeadingZero(IntegerType),
    CountTrailingZero(IntegerType),
    CountOnes(IntegerType),
    Add(MemoryType),
    Subtract(MemoryType),
    Multiply(MemoryType),
    IntDivision {
        ty: IntegerType,
        signed: bool,
    },
    FloatDivision(FloatType),
    Remainder {
        ty: IntegerType,
        signed: bool,
    },
    And(IntegerType),
    Or(IntegerType),
    Xor(IntegerType),
    ShiftLeft(IntegerType),
    ShiftRight {
        ty: IntegerType,
        signed: bool,
    },
    LeftRotation(IntegerType),
    RightRotation(IntegerType),
    Absolute(FloatType),
    Negate(FloatType),
    Ceil(FloatType),
    Floor(FloatType),
    Truncate(FloatType),
    Nearest(FloatType),
    SquareRoot(FloatType),
    Minimum(FloatType),
    Maximum(FloatType),
    CopySign(FloatType),
    IntWrap,
    // signed
    IntExtend(bool),
    IntTruncate {
        ty: IntegerType,
        float: FloatType,
        signed: bool,
    },
    Convert {
        ty: FloatType,
        int: IntegerType,
        signed: bool,
    },
    FloatDemote,
    FloatPromote,
    IntReinterpret,
    LongReinterpret,
    FloatReinterpret,
    DoubleReinterpret,
    Extend {
        ty: IntegerType,
        base: StorageType,
    },
    SaturateTruncate {
        ty: IntegerType,
        float: FloatType,
        signed: bool,
    },
}

impl Instruction {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<usize> {
        match self {
            Instruction::Unreachable => writer.write(&[0x00]),
            Instruction::NOP => writer.write(&[0x01]),
            Instruction::Block { ty, instrs } => {
                let mut length = writer.write(&[0x02])?;
                length += ty.encode(writer)?;
                for instr in instrs {
                    length += instr.encode(writer)?;
                }
                length += writer.write(&[0x0B])?;
                Ok(length)
            }
            Instruction::Loop { ty, instrs } => {
                let mut length = writer.write(&[0x03])?;
                length += ty.encode(writer)?;
                for instr in instrs {
                    length += instr.encode(writer)?;
                }
                length += writer.write(&[0x0B])?;
                Ok(length)
            }
            Instruction::If {
                ty,
                accept_instrs,
                reject_instrs,
            } => {
                let mut length = writer.write(&[0x04])?;
                length += ty.encode(writer)?;
                for instr in accept_instrs {
                    length += instr.encode(writer)?;
                }
                if let Some(reject) = reject_instrs {
                    length += writer.write(&[0x05])?;
                    for instr in reject {
                        length += instr.encode(writer)?;
                    }
                }
                length += writer.write(&[0x0B])?;
                Ok(length)
            }
            Instruction::Branch(label) => {
                let mut length = writer.write(&[0x0C])?;
                length += types::encode_u32(writer, *label)?;
                Ok(length)
            }
            Instruction::BranchIf(label) => {
                let mut length = writer.write(&[0x0D])?;
                length += types::encode_u32(writer, *label)?;
                Ok(length)
            }
            Instruction::BranchTable { labels, operand } => {
                let mut length = writer.write(&[0x0E])?;
                let mut buf = Vec::new();
                for label in labels {
                    types::encode_u32(&mut buf, *label)?;
                }
                length += types::encode_vec(writer, &buf, labels.len() as u32)?;
                length += types::encode_u32(writer, *operand)?;
                Ok(length)
            }
            Instruction::Return => writer.write(&[0x0F]),
            Instruction::Call(idx) => {
                let mut length = writer.write(&[0x10])?;
                length += types::encode_u32(writer, *idx)?;
                Ok(length)
            }
            Instruction::CallIndirect(idx) => {
                let mut length = writer.write(&[0x11])?;
                length += types::encode_u32(writer, *idx)?;
                length += writer.write(&[0x00])?;
                Ok(length)
            }
            Instruction::Drop => writer.write(&[0x1A]),
            Instruction::Select => writer.write(&[0x1B]),
            Instruction::LocalGet(idx) => {
                let mut length = writer.write(&[0x20])?;
                length += types::encode_u32(writer, *idx)?;
                Ok(length)
            }
            Instruction::LocalSet(idx) => {
                let mut length = writer.write(&[0x21])?;
                length += types::encode_u32(writer, *idx)?;
                Ok(length)
            }
            Instruction::LocalTee(idx) => {
                let mut length = writer.write(&[0x22])?;
                length += types::encode_u32(writer, *idx)?;
                Ok(length)
            }
            Instruction::GlobalGet(idx) => {
                let mut length = writer.write(&[0x23])?;
                length += types::encode_u32(writer, *idx)?;
                Ok(length)
            }
            Instruction::GlobalSet(idx) => {
                let mut length = writer.write(&[0x24])?;
                length += types::encode_u32(writer, *idx)?;
                Ok(length)
            }
            Instruction::Load { mem, ty, storage } => {
                let mut length = 0;
                match ty {
                    MemoryType::Int => {
                        if let Some(storage) = storage {
                            match storage.1 {
                                StorageType::Byte => {
                                    if storage.0 {
                                        length += writer.write(&[0x2C])?;
                                    } else {
                                        length += writer.write(&[0x2D])?;
                                    }
                                }
                                StorageType::Short => {
                                    if storage.0 {
                                        length += writer.write(&[0x2E])?;
                                    } else {
                                        length += writer.write(&[0x2F])?;
                                    }
                                }
                                StorageType::Int => panic!(),
                            }
                        } else {
                            length += writer.write(&[0x28])?;
                        }
                    }
                    MemoryType::Long => {
                        if let Some(storage) = storage {
                            match storage.1 {
                                StorageType::Byte => {
                                    if storage.0 {
                                        length += writer.write(&[0x30])?;
                                    } else {
                                        length += writer.write(&[0x31])?;
                                    }
                                }
                                StorageType::Short => {
                                    if storage.0 {
                                        length += writer.write(&[0x32])?;
                                    } else {
                                        length += writer.write(&[0x33])?;
                                    }
                                }
                                StorageType::Int => {
                                    if storage.0 {
                                        length += writer.write(&[0x34])?;
                                    } else {
                                        length += writer.write(&[0x35])?;
                                    }
                                }
                            }
                        } else {
                            length += writer.write(&[0x29])?;
                        }
                    }
                    MemoryType::Float => {
                        if let Some(_) = storage {
                            panic!()
                        } else {
                            length += writer.write(&[0x2A])?;
                        }
                    }
                    MemoryType::Double => {
                        if let Some(_) = storage {
                            panic!()
                        } else {
                            length += writer.write(&[0x2B])?;
                        }
                    }
                }
                length += mem.encode(writer)?;
                Ok(length)
            }
            Instruction::Store { mem, ty, storage } => {
                let mut length = 0;
                match ty {
                    MemoryType::Int => {
                        if let Some(storage) = storage {
                            match storage {
                                StorageType::Byte => {
                                    length += writer.write(&[0x3A])?;
                                }
                                StorageType::Short => {
                                    length += writer.write(&[0x3B])?;
                                }
                                StorageType::Int => panic!(),
                            }
                        } else {
                            length += writer.write(&[0x36])?;
                        }
                    }
                    MemoryType::Long => {
                        if let Some(storage) = storage {
                            match storage {
                                StorageType::Byte => {
                                    length += writer.write(&[0x3C])?;
                                }
                                StorageType::Short => {
                                    length += writer.write(&[0x3D])?;
                                }
                                StorageType::Int => {
                                    length += writer.write(&[0x3E])?;
                                }
                            }
                        } else {
                            length += writer.write(&[0x37])?;
                        }
                    }
                    MemoryType::Float => {
                        if let Some(_) = storage {
                            panic!();
                        } else {
                            length += writer.write(&[0x38])?;
                        }
                    }
                    MemoryType::Double => {
                        if let Some(_) = storage {
                            panic!();
                        } else {
                            length += writer.write(&[0x39])?;
                        }
                    }
                }
                length += mem.encode(writer)?;
                Ok(length)
            }
            Instruction::MemorySize => writer.write(&[0x3f, 0x00]),
            Instruction::MemoryGrow => writer.write(&[0x40, 0x00]),
            Instruction::Const(literal) => match literal {
                Literal::Int(int) => {
                    let mut length = writer.write(&[0x41])?;
                    length += types::encode_i32(writer, *int)?;
                    Ok(length)
                }
                Literal::Long(long) => {
                    let mut length = writer.write(&[0x42])?;
                    length += types::encode_i64(writer, *long)?;
                    Ok(length)
                }
                Literal::Float(float) => {
                    let mut length = writer.write(&[0x43])?;
                    length += types::encode_f32(writer, *float)?;
                    Ok(length)
                }
                Literal::Double(double) => {
                    let mut length = writer.write(&[0x44])?;
                    length += types::encode_f64(writer, *double)?;
                    Ok(length)
                }
            },
            Instruction::EqualZero(ty) => match ty {
                IntegerType::Int => writer.write(&[0x45]),
                IntegerType::Long => writer.write(&[0x50]),
            },
            Instruction::Equal(ty) => match ty {
                MemoryType::Int => writer.write(&[0x46]),
                MemoryType::Long => writer.write(&[0x51]),
                MemoryType::Float => writer.write(&[0x5B]),
                MemoryType::Double => writer.write(&[0x61]),
            },
            Instruction::NotEqual(ty) => match ty {
                MemoryType::Int => writer.write(&[0x47]),
                MemoryType::Long => writer.write(&[0x52]),
                MemoryType::Float => writer.write(&[0x5C]),
                MemoryType::Double => writer.write(&[0x62]),
            },
            Instruction::LessThanInt { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x48]),
                (IntegerType::Int, false) => writer.write(&[0x49]),
                (IntegerType::Long, true) => writer.write(&[0x53]),
                (IntegerType::Long, false) => writer.write(&[0x54]),
            },
            Instruction::GreaterThanInt { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x4A]),
                (IntegerType::Int, false) => writer.write(&[0x4B]),
                (IntegerType::Long, true) => writer.write(&[0x55]),
                (IntegerType::Long, false) => writer.write(&[0x56]),
            },
            Instruction::LessOrEqualInt { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x4C]),
                (IntegerType::Int, false) => writer.write(&[0x4D]),
                (IntegerType::Long, true) => writer.write(&[0x57]),
                (IntegerType::Long, false) => writer.write(&[0x58]),
            },
            Instruction::GreaterOrEqualInt { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x4E]),
                (IntegerType::Int, false) => writer.write(&[0x4F]),
                (IntegerType::Long, true) => writer.write(&[0x59]),
                (IntegerType::Long, false) => writer.write(&[0x5A]),
            },
            Instruction::LessThanFloat(ty) => match ty {
                FloatType::Float => writer.write(&[0x5D]),
                FloatType::Double => writer.write(&[0x63]),
            },
            Instruction::GreaterThanFloat(ty) => match ty {
                FloatType::Float => writer.write(&[0x5E]),
                FloatType::Double => writer.write(&[0x64]),
            },
            Instruction::LessOrEqualFloat(ty) => match ty {
                FloatType::Float => writer.write(&[0x5F]),
                FloatType::Double => writer.write(&[0x65]),
            },
            Instruction::GreaterOrEqualFloat(ty) => match ty {
                FloatType::Float => writer.write(&[0x60]),
                FloatType::Double => writer.write(&[0x66]),
            },
            Instruction::CountLeadingZero(ty) => match ty {
                IntegerType::Int => writer.write(&[0x67]),
                IntegerType::Long => writer.write(&[0x79]),
            },
            Instruction::CountTrailingZero(ty) => match ty {
                IntegerType::Int => writer.write(&[0x68]),
                IntegerType::Long => writer.write(&[0x7A]),
            },
            Instruction::CountOnes(ty) => match ty {
                IntegerType::Int => writer.write(&[0x69]),
                IntegerType::Long => writer.write(&[0x7B]),
            },
            Instruction::Add(ty) => match ty {
                MemoryType::Int => writer.write(&[0x6A]),
                MemoryType::Long => writer.write(&[0x7C]),
                MemoryType::Float => writer.write(&[0x92]),
                MemoryType::Double => writer.write(&[0xA0]),
            },
            Instruction::Subtract(ty) => match ty {
                MemoryType::Int => writer.write(&[0x6B]),
                MemoryType::Long => writer.write(&[0x7D]),
                MemoryType::Float => writer.write(&[0x93]),
                MemoryType::Double => writer.write(&[0xA1]),
            },
            Instruction::Multiply(ty) => match ty {
                MemoryType::Int => writer.write(&[0x6C]),
                MemoryType::Long => writer.write(&[0x7E]),
                MemoryType::Float => writer.write(&[0x94]),
                MemoryType::Double => writer.write(&[0xA2]),
            },
            Instruction::IntDivision { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x6D]),
                (IntegerType::Int, false) => writer.write(&[0x6E]),
                (IntegerType::Long, true) => writer.write(&[0x7F]),
                (IntegerType::Long, false) => writer.write(&[0x80]),
            },
            Instruction::FloatDivision(ty) => match ty {
                FloatType::Float => writer.write(&[0x95]),
                FloatType::Double => writer.write(&[0xA3]),
            },
            Instruction::Remainder { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x6F]),
                (IntegerType::Int, false) => writer.write(&[0x70]),
                (IntegerType::Long, true) => writer.write(&[0x81]),
                (IntegerType::Long, false) => writer.write(&[0x82]),
            },
            Instruction::And(ty) => match ty {
                IntegerType::Int => writer.write(&[0x71]),
                IntegerType::Long => writer.write(&[0x83]),
            },
            Instruction::Or(ty) => match ty {
                IntegerType::Int => writer.write(&[0x72]),
                IntegerType::Long => writer.write(&[0x84]),
            },
            Instruction::Xor(ty) => match ty {
                IntegerType::Int => writer.write(&[0x73]),
                IntegerType::Long => writer.write(&[0x85]),
            },
            Instruction::ShiftLeft(ty) => match ty {
                IntegerType::Int => writer.write(&[0x74]),
                IntegerType::Long => writer.write(&[0x86]),
            },
            Instruction::ShiftRight { ty, signed } => match (ty, signed) {
                (IntegerType::Int, true) => writer.write(&[0x75]),
                (IntegerType::Int, false) => writer.write(&[0x76]),
                (IntegerType::Long, true) => writer.write(&[0x87]),
                (IntegerType::Long, false) => writer.write(&[0x88]),
            },
            Instruction::LeftRotation(ty) => match ty {
                IntegerType::Int => writer.write(&[0x77]),
                IntegerType::Long => writer.write(&[0x78]),
            },
            Instruction::RightRotation(ty) => match ty {
                IntegerType::Int => writer.write(&[0x89]),
                IntegerType::Long => writer.write(&[0x8A]),
            },
            Instruction::Absolute(ty) => match ty {
                FloatType::Float => writer.write(&[0x8B]),
                FloatType::Double => writer.write(&[0x99]),
            },
            Instruction::Negate(ty) => match ty {
                FloatType::Float => writer.write(&[0x8C]),
                FloatType::Double => writer.write(&[0x9A]),
            },
            Instruction::Ceil(ty) => match ty {
                FloatType::Float => writer.write(&[0x8D]),
                FloatType::Double => writer.write(&[0x9B]),
            },
            Instruction::Floor(ty) => match ty {
                FloatType::Float => writer.write(&[0x8E]),
                FloatType::Double => writer.write(&[0x9C]),
            },
            Instruction::Truncate(ty) => match ty {
                FloatType::Float => writer.write(&[0x8F]),
                FloatType::Double => writer.write(&[0x9D]),
            },
            Instruction::Nearest(ty) => match ty {
                FloatType::Float => writer.write(&[0x90]),
                FloatType::Double => writer.write(&[0x9E]),
            },
            Instruction::SquareRoot(ty) => match ty {
                FloatType::Float => writer.write(&[0x91]),
                FloatType::Double => writer.write(&[0x9F]),
            },
            Instruction::Minimum(ty) => match ty {
                FloatType::Float => writer.write(&[0x96]),
                FloatType::Double => writer.write(&[0xA4]),
            },
            Instruction::Maximum(ty) => match ty {
                FloatType::Float => writer.write(&[0x97]),
                FloatType::Double => writer.write(&[0xA5]),
            },
            Instruction::CopySign(ty) => match ty {
                FloatType::Float => writer.write(&[0x98]),
                FloatType::Double => writer.write(&[0xA6]),
            },
            Instruction::IntWrap => writer.write(&[0xA7]),
            Instruction::IntExtend(signed) => match signed {
                true => writer.write(&[0xAC]),
                false => writer.write(&[0xAD]),
            },
            Instruction::IntTruncate { ty, float, signed } => match ty {
                IntegerType::Int => match (float, signed) {
                    (FloatType::Float, true) => writer.write(&[0xA8]),
                    (FloatType::Float, false) => writer.write(&[0xA9]),
                    (FloatType::Double, true) => writer.write(&[0xAA]),
                    (FloatType::Double, false) => writer.write(&[0xAB]),
                },
                IntegerType::Long => match (float, signed) {
                    (FloatType::Float, true) => writer.write(&[0xAE]),
                    (FloatType::Float, false) => writer.write(&[0xAF]),
                    (FloatType::Double, true) => writer.write(&[0xB0]),
                    (FloatType::Double, false) => writer.write(&[0xB1]),
                },
            },
            Instruction::Convert { ty, int, signed } => match ty {
                FloatType::Float => match (int, signed) {
                    (IntegerType::Int, true) => writer.write(&[0xB2]),
                    (IntegerType::Int, false) => writer.write(&[0xB3]),
                    (IntegerType::Long, true) => writer.write(&[0xB4]),
                    (IntegerType::Long, false) => writer.write(&[0xB5]),
                },
                FloatType::Double => match (int, signed) {
                    (IntegerType::Int, true) => writer.write(&[0xB7]),
                    (IntegerType::Int, false) => writer.write(&[0xB8]),
                    (IntegerType::Long, true) => writer.write(&[0xB9]),
                    (IntegerType::Long, false) => writer.write(&[0xBA]),
                },
            },
            Instruction::FloatDemote => writer.write(&[0xB6]),
            Instruction::FloatPromote => writer.write(&[0xBB]),
            Instruction::IntReinterpret => writer.write(&[0xBC]),
            Instruction::LongReinterpret => writer.write(&[0xBD]),
            Instruction::FloatReinterpret => writer.write(&[0xBE]),
            Instruction::DoubleReinterpret => writer.write(&[0xBF]),
            Instruction::Extend { ty, base } => match ty {
                IntegerType::Int => match base {
                    StorageType::Byte => writer.write(&[0xC0]),
                    StorageType::Short => writer.write(&[0xC1]),
                    StorageType::Int => panic!(),
                },
                IntegerType::Long => match base {
                    StorageType::Byte => writer.write(&[0xC2]),
                    StorageType::Short => writer.write(&[0xC3]),
                    StorageType::Int => writer.write(&[0xC4]),
                },
            },
            Instruction::SaturateTruncate { ty, float, signed } => {
                writer.write(&[0xFC])?;
                match ty {
                    IntegerType::Int => match (float, signed) {
                        (FloatType::Float, true) => writer.write(&[0x00]),
                        (FloatType::Float, false) => writer.write(&[0x01]),
                        (FloatType::Double, true) => writer.write(&[0x02]),
                        (FloatType::Double, false) => writer.write(&[0x03]),
                    },
                    IntegerType::Long => match (float, signed) {
                        (FloatType::Float, true) => writer.write(&[0x04]),
                        (FloatType::Float, false) => writer.write(&[0x05]),
                        (FloatType::Double, true) => writer.write(&[0x06]),
                        (FloatType::Double, false) => writer.write(&[0x07]),
                    },
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expr(pub Vec<Instruction>);

impl Expr {
    pub(crate) fn encode(&self, writer: &mut impl Write) -> io::Result<usize> {
        let mut length = 0;

        for instr in self.0.iter() {
            length += instr.encode(writer)?;
        }

        length += writer.write(&[0x0B])?;

        Ok(length)
    }
}
