use std::fmt::{Display, Formatter};
use crate::block::BlockIdParseError;
use crate::schem::common::format_size;

#[derive(Debug)]
#[allow(dead_code)]
pub enum LoadError {
    NBTReadError(fastnbt::error::Error),
    TagMissing(String),
    TagTypeMismatch {
        tag_path: String,
        expected_type: u8,
        found_type: u8,
    },
    InvalidValue {
        tag_path: String,
        error: String,
    },
    InvalidBlockId {
        id: String,
        reason: BlockIdParseError,
    },
    InvalidBlockProperty {
        tag_path: String,
        error: String,
    },
    PaletteTooLong(usize),
    BlockIndexOutOfRange {
        tag_path: String,
        index: i32,
        range: [i32; 2],
    },
    BlockPosOutOfRange {
        tag_path: String,
        pos: [i32; 3],
        range: [i32; 3],
    },
    FileOpenError(std::io::Error),
    MultipleBlockEntityInOnePos {
        pos: [i32; 3],
        latter_tag_path: String,
    },
    MultiplePendingTickInOnePos {
        pos: [i32; 3],
        latter_tag_path: String,
    },
    ConflictingIndexInPalette {
        index: u16,
        former_block_id: String,
        latter_block_id: String,
    },
    BlockDataIncomplete {
        tag_path: String,
        index: usize,
        detail: String,
    },
}

impl Display for LoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            LoadError::NBTReadError(nbterr)
            => write!(f, "NBT format broken. Detail: {}", nbterr),
            LoadError::TagMissing(tag_path) => write!(f, "Missing tag: {}", tag_path),
            LoadError::TagTypeMismatch { tag_path, expected_type, found_type }
            => write!(f, "Type of {} is invalid, expected {}, but found {}", tag_path, expected_type, found_type),
            LoadError::InvalidValue { tag_path, error }
            => write!(f, "Value of tag {} is invalid, detail: {}", tag_path, error),
            LoadError::InvalidBlockId { id, reason } => write!(f, "Invalid block id: \"{}\", detail: {}", id, reason),
            LoadError::InvalidBlockProperty { tag_path, error }
            => write!(f, "Invalid block property: tag_path = {}, detail: {}", tag_path, error),
            LoadError::PaletteTooLong(l) => write!(f, "Palette too long: {}", l),
            LoadError::BlockIndexOutOfRange { tag_path, index, range }
            => write!(f, "Block index out of range, tag: {}, index = {}, index should be in range [{}, {}]", tag_path, index, range[0], range[1]),
            LoadError::BlockPosOutOfRange { tag_path, pos, range }
            => write!(f, "Block pos out of range, tag: {}, coordinate: {}, range: {}", tag_path, format_size(pos), format_size(range)),
            LoadError::FileOpenError(err)
            => write!(f, "File open error: {}", err),
            LoadError::MultipleBlockEntityInOnePos { pos, latter_tag_path }
            => write!(f, "Multiple block entities in one {}, the latter block is defined at {}", format_size(pos), latter_tag_path),
            LoadError::MultiplePendingTickInOnePos { pos, latter_tag_path }
            => write!(f, "Multiple pending ticks in one {}, the latter block is defined at {}", format_size(pos), latter_tag_path),
            LoadError::ConflictingIndexInPalette { index, former_block_id, latter_block_id }
            => write!(f, "2 blocks have same id({}) in palette, \"{}\" and \"{}\"", index, former_block_id, latter_block_id),
            LoadError::BlockDataIncomplete { tag_path, index, detail }
            => write!(f, "The 3d block array stored in {} is incomplete, failed to decode at index {}, detail: {}", tag_path, index, detail),
        }
    }
}

#[macro_export]
macro_rules! unwrap_opt_tag {
        ($value_opt:expr,$expected_type:ident,$expected_default_ctor:expr,$tag_path:expr) => {
            if let Some(value)=$value_opt {
                if let Value::$expected_type(unwrapped)=value {
                    unwrapped
                }else {
                    return Err(LoadError::TagTypeMismatch{
                        tag_path: String::from($tag_path),
                        expected_type: id_of_nbt_tag(&Value::$expected_type($expected_default_ctor)),
                        found_type: id_of_nbt_tag(&value),
                        //TagTypeMismatchDetail::new($tag_path,Value::$expected_type($expected_default_ctor),value)
                    });
                }
            } else {
                return Err(LoadError::TagMissing(String::from($tag_path)));
            }
        };
    }
#[macro_export]
macro_rules! unwrap_tag {
        ($value:expr,$expected_type:ident,$expected_default_ctor:expr,$tag_path:expr) => {
                if let Value::$expected_type(unwrapped)=$value {
                    unwrapped
                }else {
                    return Err(LoadError::TagTypeMismatch{
                        tag_path: String::from($tag_path),
                        expected_type: id_of_nbt_tag(&Value::$expected_type($expected_default_ctor)),
                        found_type: id_of_nbt_tag(&$value),
                        //TagTypeMismatchDetail::new($tag_path,Value::$expected_type($expected_default_ctor),$value)
                    });
                }
        };
    }

#[derive(Debug)]
#[allow(dead_code)]
pub enum WriteError {
    NBTWriteError(fastnbt::error::Error),
    NegativeSize { size: [i32; 3], region_name: String },
    BlockIndexOfOfRange { r_pos: [i32; 3], block_index: u16, max_index: u16 },
    FileCreateError(std::io::Error),
    DuplicatedRegionName { name: String },
    SizeTooLarge { size: [u64; 3], max_size: [u64; 3] }
}

impl Display for WriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            WriteError::NBTWriteError(err) => write!(f, "Failed to write nbt, detail: {}", err),
            WriteError::NegativeSize { size, region_name }
            => write!(f, "region \"{}\" has negative size: {}", region_name, format_size(size)),
            WriteError::BlockIndexOfOfRange { r_pos, block_index, max_index }
            => write!(f, "Block index out of range: relative pos: {}, found block index {} but expected [0,{}]",
                      format_size(r_pos), block_index, max_index),
            WriteError::FileCreateError(err)
            => write!(f, "Failed to create file, detail: {}", err),
            WriteError::DuplicatedRegionName { name }
            => write!(f, "More than one region used name \"{}\"", name),
            WriteError::SizeTooLarge { size, max_size }
            => write!(f, "Schematic size {} exceeds maximum size {} of current format.", format_size(size), format_size(max_size)),
        }
    }
}
