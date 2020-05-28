use super::sections::*;
use super::types;
use std::io::{self, Write};
use types::ValType;

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
pub enum StorageType {
    I8,  // 8
    I16, // 16
    I32, // 32
}

#[derive(Debug, Copy, Clone)]
pub enum Literal {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, Copy, Clone)]
pub enum IntegerType {
    I32,
    I64,
}

#[derive(Debug, Copy, Clone)]
pub enum FloatType {
    F32,
    F64,
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
        ty: ValType,
        storage: Option<(bool, StorageType)>,
    },
    Store {
        mem: MemoryArgument,
        ty: ValType,
        storage: Option<StorageType>,
    },
    MemorySize,
    MemoryGrow,
    Const(Literal),
    EqualZero(IntegerType),
    Equal(ValType),
    NotEqual(ValType),
    LessThanI32 {
        ty: IntegerType,
        signed: bool,
    },
    GreaterThanI32 {
        ty: IntegerType,
        signed: bool,
    },
    LessOrEqualI32 {
        ty: IntegerType,
        signed: bool,
    },
    GreaterOrEqualI32 {
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
    Add(ValType),
    Subtract(ValType),
    Multiply(ValType),
    I32Division {
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
    I32Wrap,
    // signed
    I32Extend(bool),
    I32Truncate {
        ty: IntegerType,
        float: FloatType,
        signed: bool,
    },
    Convert {
        ty: FloatType,
        tgt_ty: IntegerType,
        signed: bool,
    },
    FloatDemote,
    FloatPromote,
    I32ReI32erpret,
    LongReI32erpret,
    FloatReI32erpret,
    DoubleReI32erpret,
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
                    ValType::I32 => {
                        if let Some(storage) = storage {
                            match storage.1 {
                                StorageType::I8 => {
                                    if storage.0 {
                                        length += writer.write(&[0x2C])?;
                                    } else {
                                        length += writer.write(&[0x2D])?;
                                    }
                                }
                                StorageType::I16 => {
                                    if storage.0 {
                                        length += writer.write(&[0x2E])?;
                                    } else {
                                        length += writer.write(&[0x2F])?;
                                    }
                                }
                                StorageType::I32 => panic!(),
                            }
                        } else {
                            length += writer.write(&[0x28])?;
                        }
                    }
                    ValType::I64 => {
                        if let Some(storage) = storage {
                            match storage.1 {
                                StorageType::I8 => {
                                    if storage.0 {
                                        length += writer.write(&[0x30])?;
                                    } else {
                                        length += writer.write(&[0x31])?;
                                    }
                                }
                                StorageType::I16 => {
                                    if storage.0 {
                                        length += writer.write(&[0x32])?;
                                    } else {
                                        length += writer.write(&[0x33])?;
                                    }
                                }
                                StorageType::I32 => {
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
                    ValType::F32 => {
                        if let Some(_) = storage {
                            panic!()
                        } else {
                            length += writer.write(&[0x2A])?;
                        }
                    }
                    ValType::F64 => {
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
                    ValType::I32 => {
                        if let Some(storage) = storage {
                            match storage {
                                StorageType::I8 => {
                                    length += writer.write(&[0x3A])?;
                                }
                                StorageType::I16 => {
                                    length += writer.write(&[0x3B])?;
                                }
                                StorageType::I32 => panic!(),
                            }
                        } else {
                            length += writer.write(&[0x36])?;
                        }
                    }
                    ValType::I64 => {
                        if let Some(storage) = storage {
                            match storage {
                                StorageType::I8 => {
                                    length += writer.write(&[0x3C])?;
                                }
                                StorageType::I16 => {
                                    length += writer.write(&[0x3D])?;
                                }
                                StorageType::I32 => {
                                    length += writer.write(&[0x3E])?;
                                }
                            }
                        } else {
                            length += writer.write(&[0x37])?;
                        }
                    }
                    ValType::F32 => {
                        if let Some(_) = storage {
                            panic!();
                        } else {
                            length += writer.write(&[0x38])?;
                        }
                    }
                    ValType::F64 => {
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
                Literal::I32(int) => {
                    let mut length = writer.write(&[0x41])?;
                    length += types::encode_i32(writer, *int)?;
                    Ok(length)
                }
                Literal::I64(long) => {
                    let mut length = writer.write(&[0x42])?;
                    length += types::encode_i64(writer, *long)?;
                    Ok(length)
                }
                Literal::F32(float) => {
                    let mut length = writer.write(&[0x43])?;
                    length += types::encode_f32(writer, *float)?;
                    Ok(length)
                }
                Literal::F64(double) => {
                    let mut length = writer.write(&[0x44])?;
                    length += types::encode_f64(writer, *double)?;
                    Ok(length)
                }
            },
            Instruction::EqualZero(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x45]),
                IntegerType::I64 => writer.write(&[0x50]),
            },
            Instruction::Equal(ty) => match ty {
                ValType::I32 => writer.write(&[0x46]),
                ValType::I64 => writer.write(&[0x51]),
                ValType::F32 => writer.write(&[0x5B]),
                ValType::F64 => writer.write(&[0x61]),
            },
            Instruction::NotEqual(ty) => match ty {
                ValType::I32 => writer.write(&[0x47]),
                ValType::I64 => writer.write(&[0x52]),
                ValType::F32 => writer.write(&[0x5C]),
                ValType::F64 => writer.write(&[0x62]),
            },
            Instruction::LessThanI32 { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x48]),
                (IntegerType::I32, false) => writer.write(&[0x49]),
                (IntegerType::I64, true) => writer.write(&[0x53]),
                (IntegerType::I64, false) => writer.write(&[0x54]),
            },
            Instruction::GreaterThanI32 { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x4A]),
                (IntegerType::I32, false) => writer.write(&[0x4B]),
                (IntegerType::I64, true) => writer.write(&[0x55]),
                (IntegerType::I64, false) => writer.write(&[0x56]),
            },
            Instruction::LessOrEqualI32 { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x4C]),
                (IntegerType::I32, false) => writer.write(&[0x4D]),
                (IntegerType::I64, true) => writer.write(&[0x57]),
                (IntegerType::I64, false) => writer.write(&[0x58]),
            },
            Instruction::GreaterOrEqualI32 { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x4E]),
                (IntegerType::I32, false) => writer.write(&[0x4F]),
                (IntegerType::I64, true) => writer.write(&[0x59]),
                (IntegerType::I64, false) => writer.write(&[0x5A]),
            },
            Instruction::LessThanFloat(ty) => match ty {
                FloatType::F32 => writer.write(&[0x5D]),
                FloatType::F64 => writer.write(&[0x63]),
            },
            Instruction::GreaterThanFloat(ty) => match ty {
                FloatType::F32 => writer.write(&[0x5E]),
                FloatType::F64 => writer.write(&[0x64]),
            },
            Instruction::LessOrEqualFloat(ty) => match ty {
                FloatType::F32 => writer.write(&[0x5F]),
                FloatType::F64 => writer.write(&[0x65]),
            },
            Instruction::GreaterOrEqualFloat(ty) => match ty {
                FloatType::F32 => writer.write(&[0x60]),
                FloatType::F64 => writer.write(&[0x66]),
            },
            Instruction::CountLeadingZero(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x67]),
                IntegerType::I64 => writer.write(&[0x79]),
            },
            Instruction::CountTrailingZero(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x68]),
                IntegerType::I64 => writer.write(&[0x7A]),
            },
            Instruction::CountOnes(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x69]),
                IntegerType::I64 => writer.write(&[0x7B]),
            },
            Instruction::Add(ty) => match ty {
                ValType::I32 => writer.write(&[0x6A]),
                ValType::I64 => writer.write(&[0x7C]),
                ValType::F32 => writer.write(&[0x92]),
                ValType::F64 => writer.write(&[0xA0]),
            },
            Instruction::Subtract(ty) => match ty {
                ValType::I32 => writer.write(&[0x6B]),
                ValType::I64 => writer.write(&[0x7D]),
                ValType::F32 => writer.write(&[0x93]),
                ValType::F64 => writer.write(&[0xA1]),
            },
            Instruction::Multiply(ty) => match ty {
                ValType::I32 => writer.write(&[0x6C]),
                ValType::I64 => writer.write(&[0x7E]),
                ValType::F32 => writer.write(&[0x94]),
                ValType::F64 => writer.write(&[0xA2]),
            },
            Instruction::I32Division { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x6D]),
                (IntegerType::I32, false) => writer.write(&[0x6E]),
                (IntegerType::I64, true) => writer.write(&[0x7F]),
                (IntegerType::I64, false) => writer.write(&[0x80]),
            },
            Instruction::FloatDivision(ty) => match ty {
                FloatType::F32 => writer.write(&[0x95]),
                FloatType::F64 => writer.write(&[0xA3]),
            },
            Instruction::Remainder { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x6F]),
                (IntegerType::I32, false) => writer.write(&[0x70]),
                (IntegerType::I64, true) => writer.write(&[0x81]),
                (IntegerType::I64, false) => writer.write(&[0x82]),
            },
            Instruction::And(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x71]),
                IntegerType::I64 => writer.write(&[0x83]),
            },
            Instruction::Or(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x72]),
                IntegerType::I64 => writer.write(&[0x84]),
            },
            Instruction::Xor(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x73]),
                IntegerType::I64 => writer.write(&[0x85]),
            },
            Instruction::ShiftLeft(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x74]),
                IntegerType::I64 => writer.write(&[0x86]),
            },
            Instruction::ShiftRight { ty, signed } => match (ty, signed) {
                (IntegerType::I32, true) => writer.write(&[0x75]),
                (IntegerType::I32, false) => writer.write(&[0x76]),
                (IntegerType::I64, true) => writer.write(&[0x87]),
                (IntegerType::I64, false) => writer.write(&[0x88]),
            },
            Instruction::LeftRotation(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x77]),
                IntegerType::I64 => writer.write(&[0x78]),
            },
            Instruction::RightRotation(ty) => match ty {
                IntegerType::I32 => writer.write(&[0x89]),
                IntegerType::I64 => writer.write(&[0x8A]),
            },
            Instruction::Absolute(ty) => match ty {
                FloatType::F32 => writer.write(&[0x8B]),
                FloatType::F64 => writer.write(&[0x99]),
            },
            Instruction::Negate(ty) => match ty {
                FloatType::F32 => writer.write(&[0x8C]),
                FloatType::F64 => writer.write(&[0x9A]),
            },
            Instruction::Ceil(ty) => match ty {
                FloatType::F32 => writer.write(&[0x8D]),
                FloatType::F64 => writer.write(&[0x9B]),
            },
            Instruction::Floor(ty) => match ty {
                FloatType::F32 => writer.write(&[0x8E]),
                FloatType::F64 => writer.write(&[0x9C]),
            },
            Instruction::Truncate(ty) => match ty {
                FloatType::F32 => writer.write(&[0x8F]),
                FloatType::F64 => writer.write(&[0x9D]),
            },
            Instruction::Nearest(ty) => match ty {
                FloatType::F32 => writer.write(&[0x90]),
                FloatType::F64 => writer.write(&[0x9E]),
            },
            Instruction::SquareRoot(ty) => match ty {
                FloatType::F32 => writer.write(&[0x91]),
                FloatType::F64 => writer.write(&[0x9F]),
            },
            Instruction::Minimum(ty) => match ty {
                FloatType::F32 => writer.write(&[0x96]),
                FloatType::F64 => writer.write(&[0xA4]),
            },
            Instruction::Maximum(ty) => match ty {
                FloatType::F32 => writer.write(&[0x97]),
                FloatType::F64 => writer.write(&[0xA5]),
            },
            Instruction::CopySign(ty) => match ty {
                FloatType::F32 => writer.write(&[0x98]),
                FloatType::F64 => writer.write(&[0xA6]),
            },
            Instruction::I32Wrap => writer.write(&[0xA7]),
            Instruction::I32Extend(signed) => match signed {
                true => writer.write(&[0xAC]),
                false => writer.write(&[0xAD]),
            },
            Instruction::I32Truncate { ty, float, signed } => match ty {
                IntegerType::I32 => match (float, signed) {
                    (FloatType::F32, true) => writer.write(&[0xA8]),
                    (FloatType::F32, false) => writer.write(&[0xA9]),
                    (FloatType::F64, true) => writer.write(&[0xAA]),
                    (FloatType::F64, false) => writer.write(&[0xAB]),
                },
                IntegerType::I64 => match (float, signed) {
                    (FloatType::F32, true) => writer.write(&[0xAE]),
                    (FloatType::F32, false) => writer.write(&[0xAF]),
                    (FloatType::F64, true) => writer.write(&[0xB0]),
                    (FloatType::F64, false) => writer.write(&[0xB1]),
                },
            },
            Instruction::Convert { ty, tgt_ty, signed } => match ty {
                FloatType::F32 => match (tgt_ty, signed) {
                    (IntegerType::I32, true) => writer.write(&[0xB2]),
                    (IntegerType::I32, false) => writer.write(&[0xB3]),
                    (IntegerType::I64, true) => writer.write(&[0xB4]),
                    (IntegerType::I64, false) => writer.write(&[0xB5]),
                },
                FloatType::F64 => match (tgt_ty, signed) {
                    (IntegerType::I32, true) => writer.write(&[0xB7]),
                    (IntegerType::I32, false) => writer.write(&[0xB8]),
                    (IntegerType::I64, true) => writer.write(&[0xB9]),
                    (IntegerType::I64, false) => writer.write(&[0xBA]),
                },
            },
            Instruction::FloatDemote => writer.write(&[0xB6]),
            Instruction::FloatPromote => writer.write(&[0xBB]),
            Instruction::I32ReI32erpret => writer.write(&[0xBC]),
            Instruction::LongReI32erpret => writer.write(&[0xBD]),
            Instruction::FloatReI32erpret => writer.write(&[0xBE]),
            Instruction::DoubleReI32erpret => writer.write(&[0xBF]),
            Instruction::Extend { ty, base } => match ty {
                IntegerType::I32 => match base {
                    StorageType::I8 => writer.write(&[0xC0]),
                    StorageType::I16 => writer.write(&[0xC1]),
                    StorageType::I32 => panic!(),
                },
                IntegerType::I64 => match base {
                    StorageType::I8 => writer.write(&[0xC2]),
                    StorageType::I16 => writer.write(&[0xC3]),
                    StorageType::I32 => writer.write(&[0xC4]),
                },
            },
            Instruction::SaturateTruncate { ty, float, signed } => {
                writer.write(&[0xFC])?;
                match ty {
                    IntegerType::I32 => match (float, signed) {
                        (FloatType::F32, true) => writer.write(&[0x00]),
                        (FloatType::F32, false) => writer.write(&[0x01]),
                        (FloatType::F64, true) => writer.write(&[0x02]),
                        (FloatType::F64, false) => writer.write(&[0x03]),
                    },
                    IntegerType::I64 => match (float, signed) {
                        (FloatType::F32, true) => writer.write(&[0x04]),
                        (FloatType::F32, false) => writer.write(&[0x05]),
                        (FloatType::F64, true) => writer.write(&[0x06]),
                        (FloatType::F64, false) => writer.write(&[0x07]),
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
