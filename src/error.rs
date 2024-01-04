use std::fmt::{Display, Formatter};

#[derive(Debug)]
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
    InvalidBlockId(String),
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
            LoadError::InvalidBlockId(id) => write!(f, "Invalid block id: \"{}\"", id),
            LoadError::InvalidBlockProperty { tag_path, error }
            => write!(f, "Invalid block property: tag_path = {}, detail: {}", tag_path, error),
            LoadError::PaletteTooLong(l) => write!(f, "Palette too long: {}", l),
            LoadError::BlockIndexOutOfRange { tag_path, index, range }
            => write!(f, "Block index out of range, tag: {}, index = {}, index should be in range [{}, {}]", tag_path, index, range[0], range[1]),
            LoadError::BlockPosOutOfRange { tag_path, pos, range }
            => write!(f, "Block pos out of range, tag: {}, coordinate: [{}, {}, {}], range: [{}, {}, {}]", tag_path, pos[0], pos[1], pos[2], range[0], range[1], range[2]),
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
pub enum WriteError {
    NBTWriteError(fastnbt::error::Error),
    NegativeSize { size: [i32; 3], region_name: String },
}

impl Display for WriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            WriteError::NBTWriteError(err) => write!(f, "Failed to write nbt, detail: {}", err),
            WriteError::NegativeSize { size, region_name }
            => write!(f, "region \"{}\" has negative size: [{}, {}, {}]", region_name, size[0], size[1], size[2]),
        }
    }
}
