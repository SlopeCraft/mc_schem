/*
mc_schem is a rust library to generate, load, manipulate and save minecraft
schematic files. Copyright (C) 2026 ToKiNoBug

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE.  See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::c_ffi::{rust_reader, rust_string_receiver, rust_writer};
use fastnbt::Value;
use std::collections::HashMap;
use std::ffi::c_char;
use std::ptr::{null_mut, slice_from_raw_parts};

//nbt_hashmap* mc_schem_create_nbt_hashmap()
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap() -> *mut HashMap<String, Value> {
    Box::into_raw(Box::from(HashMap::new()))
}

// [[nodiscard]] size_t mc_schem_nbt_hashmap_get_size(const nbt_hashmap*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_nbt_hashmap_get_size(
    ptr: *const HashMap<String, Value>,
) -> usize {
    ptr.as_ref_unchecked().len()
}

//[[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_binary( const uint8_t* buffer, size_t bytes, const rust_string_receiver* error_message_receiver);#[no_mangle]
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap_from_binary(
    buffer: *const u8,
    len: usize,
    error_msg_receiver: *const rust_string_receiver,
) -> *mut HashMap<String, Value> {
    match fastnbt::from_bytes::<HashMap<String, Value>>(std::slice::from_raw_parts(buffer, len)) {
        Err(e) => {
            error_msg_receiver
                .as_ref_unchecked()
                .receive(&e.to_string());
            null_mut()
        }
        Ok(nbt) => Box::into_raw(Box::from(nbt)),
    }
}

// [[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_binary_stream(istream_wrapper* src, const rust_string_receiver* error_message_receiver);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap_from_binary_stream(
    src: *mut rust_reader,
    error_msg_receiver: *const rust_string_receiver,
) -> *mut HashMap<String, Value> {
    let nbt: Result<HashMap<String, Value>, _> = fastnbt::from_reader(src.as_mut_unchecked());
    match nbt {
        Err(e) => {
            error_msg_receiver
                .as_ref_unchecked()
                .receive(&e.to_string());
            null_mut()
        }
        Ok(nbt) => Box::into_raw(Box::from(nbt)),
    }
}

/// Create nbt from sNBT string. Different from other APIs, here str don't have
/// to be end with \0. This aims to support std::string_view and avoid deep
/// copy, because snbt strings may be long
// [[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_snbt(const char* str, size_t len, const rust_string_receiver* error_message_receiver);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap_from_snbt(
    string: *const c_char,
    len_in_bytes: usize,
    error_msg_receiver: *const rust_string_receiver,
) -> *mut HashMap<String, Value> {
    let bytes = &*slice_from_raw_parts(string as *const u8, len_in_bytes);
    let s;
    match str::from_utf8(bytes) {
        Ok(sr) => s = sr,
        Err(e) => {
            let msg = e.to_string();
            error_msg_receiver.as_ref_unchecked().receive(&msg);
            return null_mut();
        }
    }
    let nbt: HashMap<String, Value>;
    match fastsnbt::from_str(s) {
        Ok(n) => nbt = n,
        Err(e) => {
            let msg = e.to_string();
            error_msg_receiver.as_ref_unchecked().receive(&msg);
            return null_mut();
        }
    }
    Box::into_raw(Box::new(nbt))
}

// bool mc_schem_nbt_hashmap_dump_to_binary_stream(const nbt_hashmap*, ostream_wrapper* dest, const rust_string_receiver* error_message_receiver);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_nbt_hashmap_dump_to_binary_stream(
    nbt: *const HashMap<String, Value>,
    os: *mut rust_writer,
    error_msg_receiver: *const rust_string_receiver,
) -> bool {
    let result = fastnbt::to_writer(os.as_mut_unchecked(), nbt.as_ref_unchecked());
    if let Err(e) = &result {
        error_msg_receiver
            .as_ref_unchecked()
            .receive(&e.to_string())
    }
    result.is_ok()
}
