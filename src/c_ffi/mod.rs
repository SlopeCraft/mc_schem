mod nbt_ffi;
mod schem_ffi;
mod map_ffi;

use std::collections::{BTreeMap, HashMap};
use std::ffi::{c_char, c_void};
use std::mem::size_of;
use std::ptr::{null_mut, slice_from_raw_parts};
use std::str::from_utf8_unchecked;
use fastnbt::Value;
use static_assertions as sa;
use crate::block::Block;
use crate::region::{BlockEntity, Entity, PendingTick, Region};
use crate::schem::{MetaDataIR, Schematic};

#[no_mangle]
extern "C" fn MC_SCHEM_version_string() -> *const c_char {
    return "0.1.0\0".as_ptr() as *const c_char;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_major() -> u16 {
    return 0;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_minor() -> u16 {
    return 1;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_patch() -> u16 {
    return 0;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_tweak() -> u16 {
    return 0;
}

#[repr(C, align(8))]
#[derive(Debug, Clone)]
struct SchemString {
    begin: *const c_char,
    end: *const c_char,
}

impl SchemString {
    pub fn to_u8_slice(&self) -> &[u8] {
        unsafe {
            let str_beg = self.begin;
            let str_end = self.end;
            let len = (str_end as usize) - (str_beg as usize);
            return &*slice_from_raw_parts(str_beg as *const u8, len);
        }
    }

    pub fn to_str(&self) -> &str {
        unsafe {
            return from_utf8_unchecked(self.to_u8_slice());
        }
    }
    pub fn to_string(&self) -> String {
        unsafe {
            let v = Vec::from(self.to_u8_slice());
            return String::from_utf8_unchecked(v);
        }
    }
}

sa::const_assert!(size_of::<SchemString>()==2*size_of::<usize>());

impl SchemString {
    fn new(src: &str) -> SchemString {
        unsafe {
            let begin = src.as_ptr() as *const c_char;
            return SchemString {
                begin,
                end: begin.add(src.len()),
            };
        }
    }
}

#[derive(Debug)]
#[repr(C)]
enum RsObjWrapper<T> {
    Owned(Box<T>),
    Ref(*mut T),
}

#[allow(dead_code)]
impl<T> RsObjWrapper<T> {
    const ENUM_SIZE: usize = size_of::<Self>();
    //const UNUSED: () = assert!(size_of::<Self>() == 2 * size_of::<usize>());
    fn is_null(&self) -> bool {
        //sa::const_assert!(Self::ENUM_SIZE==2*size_of::<usize>());

        if let RsObjWrapper::Ref(ptr) = self {
            return *ptr == null_mut();
        }

        return false;
    }
    fn release(&mut self) {
        *self = RsObjWrapper::Ref(null_mut());
    }

    fn get_ref(&self) -> &T {
        unsafe {
            return match self {
                RsObjWrapper::Owned(b) => &*b,
                RsObjWrapper::Ref(ptr) => &*(ptr.cast_const()),
            }
        }
    }

    fn get_mut_ref(&mut self) -> &mut T {
        unsafe {
            return match self {
                RsObjWrapper::Owned(b) => &mut *b,
                RsObjWrapper::Ref(ptr) => {
                    let ptr = *ptr;
                    &mut *ptr
                },
            }
        }
    }
}

type CNBTValue = RsObjWrapper<Value>;
sa::const_assert!(size_of::<CNBTValue>()==2*size_of::<usize>());


#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
enum CEnumNBTType {
    MC_SCHEM_nbt_type_byte = 1,
    MC_SCHEM_nbt_type_short = 2,
    MC_SCHEM_nbt_type_int = 3,
    MC_SCHEM_nbt_type_long = 4,
    MC_SCHEM_nbt_type_float = 5,
    MC_SCHEM_nbt_type_double = 6,
    MC_SCHEM_nbt_type_byte_array = 7,
    MC_SCHEM_nbt_type_string = 8,
    MC_SCHEM_nbt_type_list = 9,
    MC_SCHEM_nbt_type_compound = 10,
    MC_SCHEM_nbt_type_int_array = 11,
    MC_SCHEM_nbt_type_long_array = 12,
}

#[no_mangle]
extern "C" fn MC_SCHEM_rust_object_is_reference(value: *const CNBTValue) -> bool {
    unsafe {
        return match *value {
            RsObjWrapper::Owned(_) => false,
            RsObjWrapper::Ref(_) => true,
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_rust_object_is_null(value: *const CNBTValue) -> bool {
    unsafe {
        let value = &*value;
        return value.is_null();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_rust_object_get_null() -> RsObjWrapper<()> {
    sa::const_assert!(size_of::<RsObjWrapper<()>>()==2*size_of::<usize>());
    return RsObjWrapper::Ref(null_mut());
}

#[repr(u8)]
enum CMapRef {
    StrStr(*mut BTreeMap<String, String>),
    StrValue(*mut HashMap<String, Value>),
    PosBlockEntity(*mut HashMap<[i32; 3], BlockEntity>),
    PosPendingTick(*mut HashMap<[i32; 3], PendingTick>),
}
sa::const_assert!(size_of::<CMapRef>()==2*size_of::<usize>());

#[repr(C)]
#[derive(PartialEq)]
enum CMapRefKeyType {
    String,
    Pos,
}

#[repr(C)]
#[derive(PartialEq)]
enum CMapRefValueType {
    String,
    NBT,
    BlockEntity,
    PendingTick,
}

type CBlock = RsObjWrapper<Block>;
type CEntity = RsObjWrapper<Entity>;
type CBlockEntity = RsObjWrapper<BlockEntity>;
type CPendingTick = RsObjWrapper<PendingTick>;
type CRegion = RsObjWrapper<Region>;
type CMetaDataIR = RsObjWrapper<MetaDataIR>;
type CSchematic = RsObjWrapper<Schematic>;

