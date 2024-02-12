use std::collections::{BTreeMap, HashMap};
use std::ffi::{c_char};
use std::ptr::{null, null_mut, slice_from_raw_parts, slice_from_raw_parts_mut};
use std::str::from_utf8_unchecked;
use static_assertions as sa;
use std::mem::size_of;
use fastnbt::Value;
use crate::block::BlockIdParseError;
use crate::region::{BlockEntity, PendingTick};

mod map_ffi;
mod nbt_ffi;
mod block_ffi;
mod region_ffi;


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
#[derive(Debug, Clone, Copy)]
struct CStringView {
    begin: *const c_char,
    end: *const c_char,
}
sa::const_assert!(size_of::<CStringView>()==2*size_of::<usize>());
#[allow(dead_code)]
impl CStringView {
    pub fn from(src: &str) -> CStringView {
        unsafe {
            return CStringView {
                begin: src.as_ptr() as *const c_char,
                end: (src.as_ptr() as *const c_char).add(src.as_bytes().len()),
            };
        }
    }
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

#[no_mangle]
extern "C" fn MC_SCHEM_string_unwrap(src: *const String) -> CStringView {
    unsafe {
        let src = &*src;
        return CStringView::from(&src);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_string_set(s: *mut String, src: CStringView) {
    unsafe {
        *s = src.to_string();
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone)]
enum CMapKeyType {
    String = 0,
    Pos = 1,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone)]
enum CMapValueType {
    String = 0,
    NBT = 1,
    BlockEntity = 2,
    PendingTick = 3,
}

#[repr(C)]
enum CMapRef {
    StrStr(*mut BTreeMap<String, String>),
    StrValue(*mut HashMap<String, Value>),
    PosBlockEntity(*mut HashMap<[i32; 3], BlockEntity>),
    PosPendingTick(*mut HashMap<[i32; 3], PendingTick>),
}
sa::const_assert!(size_of::<CMapRef>()==2*size_of::<usize>());

#[repr(C)]
enum CMapBox {
    StrStr(Box<BTreeMap<String, String>>),
    StrValue(Box<HashMap<String, Value>>),
    PosBlockEntity(Box<HashMap<[i32; 3], BlockEntity>>),
    PosPendingTick(Box<HashMap<[i32; 3], PendingTick>>),
    None,
}
sa::const_assert!(size_of::<CMapBox>()==2*size_of::<usize>());


#[repr(C)]
union CMapKeyWrapper {
    string: CStringView,
    pos: [i32; 3],
}

#[repr(C)]
union CMapValueWrapper {
    string: *mut String,
    nbt: *mut Value,
    block_entity: *mut BlockEntity,
    pending_tick: *mut PendingTick,
}
sa::const_assert!(size_of::<CMapValueWrapper>()==size_of::<usize>());

pub struct KVRef<K, V> {
    pub key: *const K,
    pub value: *mut V,
}

impl<K, V> KVRef<K, V> {
    pub fn new<'a>(src: Option<(&'a K, &'a mut V)>) -> KVRef<K, V> {
        if let Some((k, v)) = src {
            return KVRef {
                key: k as *const K,
                value: v as *mut V,
            };
        }
        return KVRef { key: null(), value: null_mut() };
    }

    pub fn is_null(&self) -> bool {
        return self.key == null();
    }
}

#[repr(C)]
#[warn(improper_ctypes_definitions)]// memory layout is invisible in C
enum CMapIterator {
    StrStr {
        iter: std::collections::btree_map::IterMut<'static, String, String>,
        deref: KVRef<String, String>,
    },
    StrValue {
        iter: std::collections::hash_map::IterMut<'static, String, Value>,
        deref: KVRef<String, Value>,
    },
    PosBlockEntity {
        iter: std::collections::hash_map::IterMut<'static, [i32; 3], BlockEntity>,
        deref: KVRef<[i32; 3], BlockEntity>,
    },
    PosPendingTick {
        iter: std::collections::hash_map::IterMut<'static, [i32; 3], PendingTick>,
        deref: KVRef<[i32; 3], PendingTick>,
    },
    None,
}
sa::const_assert!(size_of::<CMapIterator>()==12*size_of::<usize>());


#[test]
fn sizes() {
    println!("Size of usize = {}", size_of::<usize>());
    println!("Size of iter_mut = {}", size_of::<std::collections::btree_map::IterMut<'static, String, String>>());
    println!("Size of CMapIterator = {}", size_of::<CMapIterator>());

    println!("Size of fastnbt::Value = {}", size_of::<Value>());

    println!("Size of CBlockError = {}", size_of::<BlockIdParseError>());
}

#[repr(C)]
#[allow(non_camel_case_types, dead_code)]
enum CEnumNBTType {
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

type CValueBox = Box<Value>;
sa::const_assert!(size_of::<CValueBox>()==size_of::<usize>());

#[repr(C)]
struct CArrayView<T> {
    begin: *mut T,
    end: *mut T,
}

impl<T> CArrayView<T> {
    pub fn from_slice(slice: &[T]) -> CArrayView<T> {
        unsafe {
            let begin = slice.as_ptr() as *mut T;
            return CArrayView {
                begin,
                end: begin.add(slice.len()),
            }
        }
    }

    pub unsafe fn to_slice(&self) -> &mut [T] {
        let len = self.end.offset_from(self.begin);
        return &mut *(slice_from_raw_parts_mut(self.begin, len as usize));
    }

    pub unsafe fn to_vec(&self) -> Vec<T>
        where T: Clone {
        return self.to_slice().to_vec();
    }

    pub fn empty() -> CArrayView<T> {
        return CArrayView {
            begin: null_mut(),
            end: null_mut(),
        }
    }
}

type CByteArrayView = CArrayView<i8>;
type CIntArrayView = CArrayView<i32>;
type CLongArrayView = CArrayView<i64>;
type CNBTListView = CArrayView<Value>;

#[repr(C)]
struct CPosInt {
    pos: [i32; 3],
}

#[repr(C)]
struct CPosDouble {
    pos: [f64; 3],
}