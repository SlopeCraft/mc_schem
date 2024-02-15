use std::fmt::{Display, Formatter};
use strum::Display;
use crate::block::{Block, BlockIdParseError};
use crate::old_block::OldBlockParseError;
use crate::region::Region;
use crate::schem::common::format_size;

#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
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
    InvalidBlockNumberId {
        tag_path: String,
        detail: OldBlockParseError,
    },
    UnrecognisedExtension {
        extension: String,
    },
    //write error
    NBTWriteError(fastnbt::error::Error),
    NegativeSize { size: [i32; 3], region_name: String },
    BlockIndexOfOfRange { r_pos: [i32; 3], block_index: u16, max_index: u16 },
    FileCreateError(std::io::Error),
    DuplicatedRegionName { name: String },
    SizeTooLarge { size: [u64; 3], max_size: [u64; 3] },
    UnsupportedVersion { data_version_i32: i32 },
    UnsupportedWorldEdit13Version {
        version: i32,
        supported_versions: Vec<i32>,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Error::NBTReadError(nbterr)
            => write!(f, "NBT format broken. Detail: {}", nbterr),
            Error::TagMissing(tag_path) => write!(f, "Missing tag: {}", tag_path),
            Error::TagTypeMismatch { tag_path, expected_type, found_type }
            => write!(f, "Type of {} is invalid, expected {}, but found {}", tag_path, expected_type, found_type),
            Error::InvalidValue { tag_path, error }
            => write!(f, "Value of tag {} is invalid, detail: {}", tag_path, error),
            Error::InvalidBlockId { id, reason } => write!(f, "Invalid block id: \"{}\", detail: {}", id, reason),
            Error::InvalidBlockProperty { tag_path, error }
            => write!(f, "Invalid block property: tag_path = {}, detail: {}", tag_path, error),
            Error::PaletteTooLong(l) => write!(f, "Palette too long: {}", l),
            Error::BlockIndexOutOfRange { tag_path, index, range }
            => write!(f, "Block index out of range, tag: {}, index = {}, index should be in range [{}, {}]", tag_path, index, range[0], range[1]),
            Error::BlockPosOutOfRange { tag_path, pos, range }
            => write!(f, "Block pos out of range, tag: {}, coordinate: {}, range: {}", tag_path, format_size(pos), format_size(range)),
            Error::FileOpenError(err)
            => write!(f, "File open error: {}", err),
            Error::MultipleBlockEntityInOnePos { pos, latter_tag_path }
            => write!(f, "Multiple block entities in one {}, the latter block is defined at {}", format_size(pos), latter_tag_path),
            Error::MultiplePendingTickInOnePos { pos, latter_tag_path }
            => write!(f, "Multiple pending ticks in one {}, the latter block is defined at {}", format_size(pos), latter_tag_path),
            Error::ConflictingIndexInPalette { index, former_block_id, latter_block_id }
            => write!(f, "2 blocks have same id({}) in palette, \"{}\" and \"{}\"", index, former_block_id, latter_block_id),
            Error::BlockDataIncomplete { tag_path, index, detail }
            => write!(f, "The 3d block array stored in {} is incomplete, failed to decode at index {}, detail: {}", tag_path, index, detail),
            Error::InvalidBlockNumberId { tag_path, detail }
            => write!(f, "Invalid number id at {tag_path}, detail: {detail}"),
            Error::UnrecognisedExtension { extension }
            => write!(f, "Unrecognised extension {extension}, can not deduce schematic format from filename extension, try loading with explicit format."),

            Error::NBTWriteError(err) => write!(f, "Failed to write nbt, detail: {}", err),
            Error::NegativeSize { size, region_name }
            => write!(f, "region \"{}\" has negative size: {}", region_name, format_size(size)),
            Error::BlockIndexOfOfRange { r_pos, block_index, max_index }
            => write!(f, "Block index out of range: relative pos: {}, found block index {} but expected [0,{}]",
                      format_size(r_pos), block_index, max_index),
            Error::FileCreateError(err)
            => write!(f, "Failed to create file, detail: {}", err),
            Error::DuplicatedRegionName { name }
            => write!(f, "More than one region used name \"{}\"", name),
            Error::SizeTooLarge { size, max_size }
            => write!(f, "Schematic size {} exceeds maximum size {} of current format.", format_size(size), format_size(max_size)),
            Error::UnsupportedVersion { data_version_i32 }
            => write!(f, "Data version {data_version_i32} is not supported."),
            Error::UnsupportedWorldEdit13Version { version, supported_versions }
            => write!(f, "World edit format version(not minecraft version) {version} is not supported, supported versions: {supported_versions:?}"),

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
                    return Err(Error::TagTypeMismatch{
                        tag_path: String::from($tag_path),
                        expected_type: id_of_nbt_tag(&Value::$expected_type($expected_default_ctor)),
                        found_type: id_of_nbt_tag(&value),
                        //TagTypeMismatchDetail::new($tag_path,Value::$expected_type($expected_default_ctor),value)
                    });
                }
            } else {
                return Err(Error::TagMissing(String::from($tag_path)));
            }
        };
    }
#[macro_export]
macro_rules! unwrap_tag {
        ($value:expr,$expected_type:ident,$expected_default_ctor:expr,$tag_path:expr) => {
                if let Value::$expected_type(unwrapped)=$value {
                    unwrapped
                }else {
                    return Err(Error::TagTypeMismatch{
                        tag_path: String::from($tag_path),
                        expected_type: id_of_nbt_tag(&Value::$expected_type($expected_default_ctor)),
                        found_type: id_of_nbt_tag(&$value),
                        //TagTypeMismatchDetail::new($tag_path,Value::$expected_type($expected_default_ctor),$value)
                    });
                }
        };
    }

#[repr(u8)]
#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum ErrorHandleResult<T> {
    HandledWithoutWarning(T),
    HandledWithWarning(T),
    NotHandled,
}

#[allow(dead_code)]
impl<T> ErrorHandleResult<T> {
    pub fn has_value(&self) -> bool {
        return if let ErrorHandleResult::NotHandled = self {
            false
        } else { true };
    }

    pub fn has_warning(&self) -> bool {
        return if let ErrorHandleResult::HandledWithWarning(_) = self {
            true
        } else { false };
    }

    pub fn to_option(self) -> Option<T> {
        return match self {
            ErrorHandleResult::NotHandled => None,
            ErrorHandleResult::HandledWithWarning(val) => Some(val),
            ErrorHandleResult::HandledWithoutWarning(val) => Some(val),
        }
    }
}

#[allow(dead_code)]
pub enum BlockPosOutOfRangeFixMethod {
    IgnoreThisBlock,
    FixPos([i32; 3]),
}

pub trait ErrorHandler {
    // returns the fixed block index
    fn fix_block_index_out_of_range(
        _region: &mut Region,
        _error: &Error) -> ErrorHandleResult<u16> {

        return ErrorHandleResult::NotHandled;
    }

    fn fix_block_pos_out_of_range(_region: &mut Region, _error: &Error) -> ErrorHandleResult<BlockPosOutOfRangeFixMethod> {
        return ErrorHandleResult::NotHandled;
    }

    fn fix_invalid_block_id(_region: &mut Region, _error: &Error) -> ErrorHandleResult<Block> {
        return ErrorHandleResult::NotHandled;
    }
}

pub struct StrictErrorHandler {}

impl ErrorHandler for StrictErrorHandler {}

pub struct DefaultErrorHandler {}

impl ErrorHandler for DefaultErrorHandler {
    fn fix_block_index_out_of_range(
        region: &mut Region,
        error: &Error) -> ErrorHandleResult<u16> {
        if let Error::BlockIndexOutOfRange { .. } = error {
            let air_id = region.find_or_append_to_palette(&Block::air());
            return ErrorHandleResult::HandledWithWarning(air_id);
        }
        return ErrorHandleResult::NotHandled;
    }

    fn fix_block_pos_out_of_range(_region: &mut Region, _error: &Error) -> ErrorHandleResult<BlockPosOutOfRangeFixMethod> {
        return ErrorHandleResult::HandledWithWarning(BlockPosOutOfRangeFixMethod::IgnoreThisBlock);
    }

    // fn fix_invalid_block_id(_region: &mut Region, _error: &LoadError) -> ErrorHandleResult<Block> {
    //     return ErrorHandleResult::NotHandled;
    // }
}