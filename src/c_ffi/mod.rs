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

use crate::block::{Block, BlockIdParseError};
use std::ffi::{c_char, c_void, CStr};
use std::ptr::null_mut;
use Box;

#[no_mangle]
pub extern "C" fn mc_schem_create_block() -> *mut Block {
    let ret = Box::new(Block::new());

    Box::into_raw(ret)
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_block(ptr: *mut Block) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[repr(C)]
pub struct rust_string_receiver {
    func_receive_string: extern "C" fn(*const u8, usize, *mut c_void),
    custom_data: *mut c_void,
}

impl rust_string_receiver {
    pub unsafe fn receive(&self, string: &str) {
        (self.func_receive_string)(string.as_ptr(), string.len(), self.custom_data);
    }
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_get_id(
    ptr: *const Block,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((*ptr).id.as_str());
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_get_namespace(
    ptr: *const Block,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((*ptr).namespace.as_str());
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_set_id(ptr: *mut Block, str: *const c_char) {
    let cstr = CStr::from_ptr(str);
    (*ptr).id = cstr.to_string_lossy().to_string();
}
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_set_namespace(ptr: *mut Block, str: *const c_char) {
    let cstr = CStr::from_ptr(str);
    (*ptr).namespace = cstr.to_string_lossy().to_string();
}
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_get_full_id(
    ptr: *const Block,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((*ptr).full_id().as_str());
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_reset(
    ptr: *mut Block,
    full_id_str: *const c_char,
    detail_nullable: *mut BlockIdParseError,
) -> bool {
    let result = Block::from_id(&CStr::from_ptr(full_id_str).to_string_lossy());

    match result {
        Ok(new_blk) => {
            *ptr = new_blk;
            true
        }
        Err(e) => {
            if detail_nullable != null_mut() {
                *detail_nullable = e;
            }
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_visit_attributes(
    ptr: *const Block,
    callback: extern "C" fn(
        key: *const u8,
        key_bytes: usize,
        value: *const u8,
        value_bytes: usize,
        custom_data: *mut c_void,
    ),
    custom_data: *mut c_void,
) {
    for (key, value) in &(*ptr).attributes {
        callback(
            key.as_ptr(),
            key.len(),
            value.as_ptr(),
            value.len(),
            custom_data,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_erase_attribute(ptr: *mut Block, key_c: *const c_char) {
    let key = CStr::from_ptr(key_c).to_string_lossy().to_string();
    let _ = (*ptr).attributes.remove(&key);
}
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_set_attribute(
    ptr: *mut Block,
    key_c: *const c_char,
    value_c: *const c_char,
) {
    let key = CStr::from_ptr(key_c).to_string_lossy().to_string();
    let value = CStr::from_ptr(value_c).to_string_lossy().to_string();
    (*ptr).attributes.insert(key, value);
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_is_air(ptr: *const Block) -> bool {
    (*ptr).is_air()
}
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_is_structure_void(ptr: *const Block) -> bool {
    (*ptr).is_structure_void()
}
