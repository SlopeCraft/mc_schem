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

use std::cmp::min;
use std::collections::{BTreeMap, HashMap};
use std::ffi::{c_char, c_void, CStr, CString};
use std::fmt::{Debug, Display, Formatter};
use std::intrinsics::copy_nonoverlapping;
use std::io::{ErrorKind, Read, Write};
use std::ptr::{drop_in_place, null, null_mut, slice_from_raw_parts, slice_from_raw_parts_mut};
use std::str::from_utf8_unchecked;
use static_assertions as sa;
use std::mem::{size_of, swap};
use fastnbt::Value;
use flate2::Compression;
#[allow(unused_imports)]
use crate::block::{CommonBlock, Block, BlockIdParseError};
use crate::error::Error;
#[allow(unused_imports)]
use crate::region::{BlockEntity, Entity, PendingTick};
use crate::schem::{Schematic, LitematicaLoadOption, VanillaStructureLoadOption, WorldEdit13LoadOption, WorldEdit12LoadOption, DataVersion, LitematicaSaveOption, VanillaStructureSaveOption, WorldEdit13SaveOption, MetaDataIR};

mod map_ffi;
mod nbt_ffi;
mod block_ffi;
mod region_ffi;
mod schem_ffi;


#[no_mangle]
extern "C" fn MC_SCHEM_version_string() -> *const c_char {
    return "1.0.0\0".as_ptr() as *const c_char;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_major() -> u16 {
    return 1;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_minor() -> u16 {
    return 0;
}

#[no_mangle]
extern "C" fn MC_SCHEM_version_patch() -> u16 {
    return 0;
}

// #[no_mangle]
// extern "C" fn MC_SCHEM_version_tweak() -> u16 {
//     return 0;
// }

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
unsafe extern "C" fn MC_SCHEM_swap_string(a: *mut String, b: *mut String) {
    swap(&mut *a, &mut *b);
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

    println!("Size of Block = {}", size_of::<Block>());
    println!("Size of Entity = {}", size_of::<Entity>());
    println!("Size of (u8,u8) = {}", size_of::<(u8, u8)>());

    println!("Size of CLitematicaLoadOption = {}", size_of::<CLitematicaLoadOption>());
    println!("Size of CVanillaStructureLoadOption = {}", size_of::<CVanillaStructureLoadOption>());
    println!("Size of CWE13LoadOption = {}", size_of::<CWE13LoadOption>());
    println!("Size of CWE12LoadOption = {}", size_of::<CWE12LoadOption>());
}

#[repr(u8)]
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

#[repr(u8)]
enum CPendingTickType {
    Fluid = 0,
    Block = 1,
}

#[repr(C)]
#[allow(dead_code)]
struct CNumberId {
    id: u8,
    damage: u8,
}

#[repr(C)]
struct CRegionBlockInfo {
    block_index: u16,
    block: *const Block,
    block_entity: *mut BlockEntity,
    pending_tick: *mut PendingTick,
}

impl Default for CRegionBlockInfo {
    fn default() -> Self {
        return CRegionBlockInfo {
            block_index: u16::MAX,
            block: null(),
            block_entity: null_mut(),
            pending_tick: null_mut(),
        }
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_release_error(b: *mut Box<Error>) {
    drop_in_place(b);
}


#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_error(a: *mut Error, b: *mut Error) {
    swap(&mut *a, &mut *b);
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_error_to_string(error: *const Error, dest: *mut c_char, capacity: usize, length: *mut usize) {
    let mut s = (*error).to_string();
    s.push('\0');
    if capacity < s.len() {
        *length = 0;
        return;
    }
    let required_bytes = s.as_bytes().len();

    *length = required_bytes;

    if capacity < required_bytes {
        return;
    }
    copy_nonoverlapping(s.as_ptr() as *const c_char, dest, s.as_bytes().len());
}

pub fn error_to_box(err: Option<Error>) -> Option<Box<Error>> {
    sa::const_assert!(size_of::<Option<Box<Error>>>()==size_of::<usize>());
    return if let Some(e) = err {
        Some(Box::from(e))
    } else {
        None
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_error_test_none() -> Option<Box<Error>> {
    return None;
}

#[no_mangle]
extern "C" fn MC_SCHEM_error_test_some() -> Option<Box<Error>> {
    return error_to_box(Some(Error::UnsupportedVersion { data_version_i32: 0 }));
}


type ReadCallback = extern "C" fn(handle: *mut c_void, buffer: *mut u8, buffer_size: usize,
                                  ok: *mut bool, error: *mut c_char, error_capacity: usize) -> usize;

#[repr(C)]
struct CReader {
    handle: *mut c_void,
    read_fun: ReadCallback,
}

#[derive(Debug)]
struct CReaderError(CString);

impl Display for CReaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let cs: &CReaderError = self;
        return std::fmt::Display::fmt(&cs, f);
    }
}

impl std::error::Error for CReaderError {}

unsafe impl Send for CReaderError {}

unsafe impl Sync for CReaderError {}

impl Read for CReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut error_msg = [0 as c_char; 1024];
        let mut ok = false;
        let read_bytes;
        read_bytes = (self.read_fun)(self.handle, buf.as_mut_ptr(), buf.len(),
                                     &mut ok as *mut bool, error_msg.as_mut_ptr(), error_msg.len());


        return if ok {
            Ok(read_bytes)
        } else {
            unsafe {
                let c_str = CStr::from_ptr(error_msg.as_ptr());
                let s = CString::from(c_str);
                Err(std::io::Error::new(ErrorKind::Other,
                                        Box::new(CReaderError(s))).into())
            }
        }
    }
}


#[repr(C)]
struct CSchemLoadResult {
    schematic: Option<Box<Schematic>>,
    error: Option<Box<Error>>,
}

impl CSchemLoadResult {
    pub fn new<T>(src: Result<(Schematic, T), Error>) -> CSchemLoadResult {
        return match src {
            Ok((s, _)) => CSchemLoadResult { schematic: Some(Box::new(s)), error: None },
            Err(e) => CSchemLoadResult { schematic: None, error: Some(Box::new(e)) },
        }
    }
}

impl<T, U> From<Result<(Schematic, T, U), Error>> for CSchemLoadResult {
    fn from(value: Result<(Schematic, T, U), Error>) -> Self {
        return match value {
            Ok((s, ..)) => CSchemLoadResult { schematic: Some(Box::new(s)), error: None },
            Err(e) => CSchemLoadResult { schematic: None, error: Some(Box::new(e)) },
        }
    }
}


#[repr(C, align(512))]
struct CLitematicaLoadOption {
    reserved: [u8; 512]
}

sa::const_assert!(size_of::<CLitematicaLoadOption>() == 512);
impl CLitematicaLoadOption {
    pub fn to_option(&self) -> LitematicaLoadOption {
        return LitematicaLoadOption {};
    }

    pub fn from_option(_src: &LitematicaLoadOption) -> Self {
        return Self { reserved: [0; 512] };
    }
}


#[repr(C, align(512))]
struct CVanillaStructureLoadOption {
    pub background_block: CommonBlock,
}

sa::const_assert!(size_of::<CVanillaStructureLoadOption>()==512);
impl CVanillaStructureLoadOption {
    pub fn to_option(&self) -> VanillaStructureLoadOption {
        return VanillaStructureLoadOption { background_block: self.background_block };
    }
    pub fn from_option(src: &VanillaStructureLoadOption) -> Self {
        return Self {
            background_block: src.background_block,
        };
    }
}


#[repr(C, align(512))]
struct CWE13LoadOption {
    reserved: [u8; 512]
}

sa::const_assert!(size_of::<CWE13LoadOption>()==512);
impl CWE13LoadOption {
    pub fn to_option(&self) -> WorldEdit13LoadOption {
        return WorldEdit13LoadOption {};
    }

    pub fn from_option(_src: &WorldEdit13LoadOption) -> Self {
        return Self { reserved: [0; 512] };
    }
}


#[repr(C, align(512))]
struct CWE12LoadOption {
    pub data_version: DataVersion
}
sa::const_assert!(size_of::<CWE12LoadOption>()==512);

impl CWE12LoadOption {
    pub fn to_option(&self) -> WorldEdit12LoadOption {
        return WorldEdit12LoadOption {
            data_version: self.data_version
        };
    }
    pub fn from_option(src: &WorldEdit12LoadOption) -> Self {
        return Self {
            data_version: src.data_version
        }
    }
}

type CWriterWriterFun = extern "C" fn(handle: *mut c_void, buffer: *const u8, buffer_size: usize,
                                      ok: *mut bool, error: *mut c_char, error_capacity: usize) -> usize;
type CWriterFlushFun = extern "C" fn(handle: *mut c_void, ok: *mut bool, error: *mut c_char, error_capacity: usize);

#[repr(C)]
struct CWriter {
    handle: *mut c_void,
    write_fun: CWriterWriterFun,
    flush_fun: CWriterFlushFun,
}

impl Write for CWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut error_msg = [0 as c_char; 1024];
        let mut ok = false;
        let write_bytes = (self.write_fun)(self.handle, buf.as_ptr(), buf.len(),
                                           &mut ok, error_msg.as_mut_ptr(), error_msg.len());
        return if ok {
            Ok(write_bytes)
        } else {
            unsafe {
                let c_str = CStr::from_ptr(error_msg.as_ptr());
                let s = CString::from(c_str);
                Err(std::io::Error::new(ErrorKind::Other,
                                        Box::new(CReaderError(s))).into())
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut error_msg = [0 as c_char; 1024];
        let mut ok = false;
        (self.flush_fun)(self.handle, &mut ok, error_msg.as_mut_ptr(), error_msg.len());
        return if ok {
            Ok(())
        } else {
            unsafe {
                let c_str = CStr::from_ptr(error_msg.as_ptr());
                let s = CString::from(c_str);
                Err(std::io::Error::new(ErrorKind::Other,
                                        Box::new(CReaderError(s))).into())
            }
        }
    }
}

#[repr(C, align(512))]
struct CLitematicaSaveOption {
    compress_level: u32,
    rename_duplicated_regions: bool,
}
sa::const_assert!(size_of::<CLitematicaSaveOption>()==512);

impl CLitematicaSaveOption {
    pub fn to_option(&self) -> LitematicaSaveOption {
        return LitematicaSaveOption {
            compress_level: Compression::new(min(self.compress_level, 9)),
            rename_duplicated_regions: self.rename_duplicated_regions,
        };
    }

    pub fn from_option(src: &LitematicaSaveOption) -> Self {
        return CLitematicaSaveOption {
            compress_level: src.compress_level.level(),
            rename_duplicated_regions: src.rename_duplicated_regions,
        };
    }
}

#[repr(C, align(512))]
struct CVanillaStructureSaveOption {
    compress_level: u32,
    keep_air: bool,
}
sa::const_assert!(size_of::<CVanillaStructureSaveOption>()==512);

impl CVanillaStructureSaveOption {
    pub fn to_option(&self) -> VanillaStructureSaveOption {
        return VanillaStructureSaveOption {
            compress_level: Compression::new(min(self.compress_level, 9)),
            keep_air: self.keep_air,
        }
    }
    pub fn from_option(src: &VanillaStructureSaveOption) -> Self {
        return CVanillaStructureSaveOption {
            compress_level: src.compress_level.level(),
            keep_air: src.keep_air,
        }
    }
}

#[repr(C, align(512))]
struct CWE13SaveOption {
    compress_level: u32,
    background_block: CommonBlock,
}
sa::const_assert!(size_of::<CWE13SaveOption>()==512);

impl CWE13SaveOption {
    pub fn to_option(&self) -> WorldEdit13SaveOption {
        return WorldEdit13SaveOption {
            compress_level: Compression::new(min(self.compress_level, 9)),
            background_block: self.background_block,
        };
    }

    pub fn from_option(src: &WorldEdit13SaveOption) -> Self {
        return CWE13SaveOption {
            compress_level: src.compress_level.level(),
            background_block: src.background_block,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
struct COption<T> {
    value: T,
    has_value: bool,
}

impl<T> COption<T>
    where T: Clone {
    pub fn to_option(&self) -> Option<T> {
        return if self.has_value {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

impl<T> From<Option<T>> for COption<T>
    where T: Default {
    fn from(src: Option<T>) -> Self {
        return match src {
            Some(val) => COption { value: val, has_value: true },
            None => COption { value: T::default(), has_value: false },
        }
    }
}

impl From<&Option<String>> for COption<CStringView> {
    fn from(src: &Option<String>) -> Self {
        return match src {
            Some(s) => COption { value: CStringView::from(&s), has_value: true },
            None => COption { value: CStringView::from(""), has_value: false },
        }
    }
}

impl COption<CStringView> {
    pub fn to_option_string(&self) -> Option<String> {
        return if self.has_value {
            Some(self.value.to_string())
        } else {
            None
        }
    }
}

#[repr(C, align(1024))]
struct CMetadata {
    pub mc_data_version: i32,

    pub time_created: i64,
    pub time_modified: i64,
    pub author: CStringView,
    pub name: CStringView,
    pub description: CStringView,

    pub litematica_version: i32,
    pub litematica_subversion: COption<i32>,

    pub schem_version: i32,
    pub schem_offset: [i32; 3],
    pub schem_we_offset: COption<[i32; 3]>,

    //pub date: COption<i64>,

    pub schem_world_edit_version: COption<CStringView>,
    pub schem_editing_platform: COption<CStringView>,
    pub schem_origin: COption<[i32; 3]>,
    pub schem_material: CStringView,
}
sa::const_assert!(size_of::<CMetadata>()==1024);

impl CMetadata {
    pub fn new(src: &MetaDataIR) -> Self {
        return CMetadata {
            mc_data_version: src.mc_data_version,
            time_created: src.time_created,
            time_modified: src.time_modified,
            author: CStringView::from(&src.author),
            name: CStringView::from(&src.name),
            description: CStringView::from(&src.name),

            litematica_version: src.litematica_version,
            litematica_subversion: COption::from(src.litematica_subversion),

            schem_version: src.schem_version,
            schem_offset: src.schem_offset,
            schem_we_offset: COption::from(src.schem_we_offset),

            //date: COption::from(src.date),

            schem_world_edit_version: COption::from(&src.schem_world_edit_version),
            schem_editing_platform: COption::from(&src.schem_editing_platform),
            schem_origin: COption::from(src.schem_origin),
            schem_material: CStringView::from(&src.schem_material),

        };
    }

    pub fn to_metadata(&self) -> MetaDataIR {
        return MetaDataIR {
            mc_data_version: self.mc_data_version,
            time_created: self.time_created,
            time_modified: self.time_modified,
            author: self.author.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),

            litematica_version: self.litematica_version,
            litematica_subversion: self.litematica_subversion.to_option(),

            schem_version: self.schem_version,
            schem_offset: self.schem_offset,
            schem_we_offset: self.schem_we_offset.to_option(),

            //date: self.date.to_option(),

            schem_world_edit_version: self.schem_world_edit_version.to_option_string(),
            schem_editing_platform: self.schem_editing_platform.to_option_string(),
            schem_origin: self.schem_origin.to_option(),
            schem_material: self.schem_material.to_string(),
        };
    }
}

pub unsafe fn write_to_c_buffer<T>(src: &[T], dest_size: *mut usize, dest: *mut T, dest_capacity: usize)
    where T: Copy {
    *dest_size = src.len();
    if dest_capacity >= src.len() {
        for (idx, t) in src.iter().enumerate() {
            *(dest.clone().add(idx)) = *t;
        }
    }
}