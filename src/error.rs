/*
mc_schem is a rust library to generate, load, manipulate and save minecraft schematic files.
Copyright (C) 2024  joseph

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::fmt::{Display, Formatter};
use strum::Display;
use crate::block::{Block, BlockIdParseError};
use crate::old_block::OldBlockParseError;
use crate::region::Region;
use crate::schem::common::format_size;

/// Errors when loading and saving schematic
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
    PaletteIsEmpty {
        tag_path: String
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
        lower_bound: [i32; 3],
        upper_bound: [i32; 3],
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
    BlockIndexOutOfRangeWriting { r_pos: [i32; 3], block_index: u16, max_index: u16 },
    FileCreateError(std::io::Error),
    DuplicatedRegionName { name: String },
    SizeTooLarge { size: [u64; 3], max_size: [u64; 3] },
    UnsupportedVersion { data_version_i32: i32 },
    UnsupportedWorldEdit13Version {
        version: i32,
        supported_versions: Vec<i32>,
    },
    IncompleteSegmentInMCA {
        bytes: usize,
    },
    InvalidSegmentRangeInMCA {
        chunk_local_x: i32,
        chunk_local_z: i32,
        offset_by_segment: u32,
        num_segments: u32,
        total_segments: usize,
    },
    InvalidMCACompressType {
        compress_label: u8,
    },
    IOReadError(std::io::Error),
    SevenZipDecompressError(sevenz_rust::Error),
    NoSuchFile {
        filename: String,
        expected_to_exist_in: String,
    },
    InvalidBiome {
        tag_path: String,
        biome: String,
    },
    InvalidChunkStatus {
        tag_path: String,
        chunk_status: String,
    },
    MissingSubChunk {
        tag_path: String,
        sub_chunk_y: Vec<i8>,
    },
    MissingMCCFile {
        filename: String,
        detail: Box<Error>,
    }
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
            Error::PaletteIsEmpty { tag_path }
            => write!(f, "Palette stored in tag {tag_path} is empty, the region can not have any blocks"),
            Error::PaletteTooLong(l) => write!(f, "Palette too long: {}", l),
            Error::BlockIndexOutOfRange { tag_path, index, range }
            => write!(f, "Block index out of range, tag: {}, index = {}, index should be in range [{}, {}]", tag_path, index, range[0], range[1]),
            Error::BlockPosOutOfRange { tag_path, pos, lower_bound, upper_bound }
            => write!(f, "Block pos out of range, tag: {}, coordinate: {}, lower bound: {}, upper bound:{}", tag_path, format_size(pos), format_size(lower_bound), format_size(upper_bound)),
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
            Error::BlockIndexOutOfRangeWriting { r_pos, block_index, max_index }
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
            Error::IncompleteSegmentInMCA { bytes }
            => write!(f, "Incomplete chunks in MCA file, the file has {bytes} bytes, but it should be multiples of 4096."),
            Error::InvalidSegmentRangeInMCA { chunk_local_x, chunk_local_z, offset_by_segment, num_segments, total_segments }
            => write!(f, "Chunk ({chunk_local_x}, {chunk_local_z}) is stored in [{offset_by_segment}\
            , {}) segments, the file has {total_segments} segments, the range is invalid",
                      offset_by_segment + num_segments),
            Error::InvalidMCACompressType { compress_label }
            => write!(f, "Invalid compress type {compress_label}, valid values: [1, 2, 3, 128, 129, 130]"),
            Error::IOReadError(e)
            => write!(f, "IOReadError, detail: {e}"),
            Error::SevenZipDecompressError(e7z)
            => write!(f, "7z decompress failed, detail: {e7z}"),
            Error::NoSuchFile { filename, expected_to_exist_in }
            => write!(f, "File \"{filename}\" doesn't exist in \"{expected_to_exist_in}\""),
            Error::InvalidBiome { tag_path, biome }
            => write!(f, "Invalid biome \"{biome}\" at {tag_path}"),
            Error::InvalidChunkStatus { tag_path, chunk_status }
            => write!(f, "Invalid chunk status \"{chunk_status}\" at {tag_path}"),
            Error::MissingSubChunk { tag_path, sub_chunk_y }
            => write!(f, "Chunk {tag_path} has missing sub chunk(s): {:#?}", sub_chunk_y),
            Error::MissingMCCFile { filename, detail }
            => write!(f, "MCC file {filename} is missing, detail: {detail}"),
        }
    }
}
/// Unwrap a `Option<&Value>` or `Option<&mut Value>` as some type, if the option is `None`, returns
/// `Err(Error::TagMissing)`. If the option is not none, but the type doesn't match, returns
/// `Err(Error::TagTypeMismatch)`.
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


/// Unwrap a `Value` as some type, otherwise return `Err(Error::TagTypeMismatch)`.
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

/// Not used now.
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

/// Not used now.
#[allow(dead_code)]
pub enum BlockPosOutOfRangeFixMethod {
    IgnoreThisBlock,
    FixPos([i32; 3]),
}

/// Not used now.
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

/// Not used now.
pub struct StrictErrorHandler {}

impl ErrorHandler for StrictErrorHandler {}

/// Not used now.
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