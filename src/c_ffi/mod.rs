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
mod block;
mod entity;
mod error;
mod meta_data;
mod region;
mod nbt_tags;
mod schematic;

use crate::block::Block;
use crate::error::Error;
use crate::region::{BlockEntity, HasPalette, PendingTick};
use crate::schem::{MetaDataIR, Schematic};
use crate::{Entity, Region};
use fastnbt::Value;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use Box;

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

#[repr(C)]
pub struct istream_wrapper {
    func_read: extern "C" fn(*mut u8, usize, *mut bool, *mut c_char, usize, *mut c_void) -> usize,
    custom_data: *mut c_void,
}

impl std::io::Read for istream_wrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut ok = false;
        let mut error_message_buffer = ['\0' as c_char; 4096];

        unsafe {
            let bytes = (self.func_read)(buf.as_mut_ptr(), buf.len(), &mut ok, error_message_buffer.as_mut_ptr(), error_message_buffer.len() - 1, self.custom_data);
            if ok {
                return Ok(bytes);
            }
            let error_msg = CStr::from_ptr(error_message_buffer.as_ptr()).to_string_lossy().to_string();
            Err(std::io::Error::other(error_msg))
        }
    }
}

#[repr(C)]
pub struct ostream_wrapper {
    func_write: extern "C" fn(*const u8, usize, *mut bool, *mut c_char, usize, *mut c_void) -> usize,
    func_flush: extern "C" fn(*mut c_void, *mut c_char, usize) -> bool,
    custom_data: *mut c_void,
}

impl std::io::Write for ostream_wrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut ok = false;
        let mut error_message_buffer = ['\0' as c_char; 4096];

        unsafe {
            let bytes = (self.func_write)(buf.as_ptr(), buf.len(), &mut ok, error_message_buffer.as_mut_ptr(), error_message_buffer.len() - 1, self.custom_data);
            if ok {
                return Ok(bytes);
            }
            let error_msg = CStr::from_ptr(error_message_buffer.as_ptr()).to_string_lossy().to_string();
            Err(std::io::Error::other(error_msg))
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut error_message_buffer = ['\0' as c_char; 4096];

        unsafe {
            let ok = (self.func_flush)(self.custom_data, error_message_buffer.as_mut_ptr(), error_message_buffer.len() - 1);
            if ok {
                return Ok(());
            }
            let error_msg = CStr::from_ptr(error_message_buffer.as_ptr()).to_string_lossy().to_string();
            Err(std::io::Error::other(error_msg))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_block(ptr: *mut Block) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_error(err: *mut Error) {
    if err.is_null() {
        return;
    }
    let _ = Box::from_raw(err);
}
#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_entity(ptr: *mut Entity) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_block_entity(ptr: *mut BlockEntity) {
    if ptr.is_null() {
        return;
    }
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_region(region: *mut Region) {
    let _ = Box::from_raw(region);
}
//void mc_schem_destroy_pending_tick(pending_tick* tick);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_pending_tick(ptr: *mut PendingTick) {
    let _ = Box::from_raw(ptr);
}
// void mc_schem_destroy_schematic(schematic* schematic);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_schematic(ptr: *mut Schematic) {
    let _ = Box::from_raw(ptr);
}
// void mc_schem_destroy_meta_data_ir(meta_data_ir* mdata);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_meta_data_ir(ptr: *mut MetaDataIR) {
    let _ = Box::from_raw(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_nbt_hashmap(ptr: *mut HashMap<String, Value>) {
    let _ = Box::from_raw(ptr);
}

// [[nodiscard]] block* mc_schem_clone_block(const block*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_block(ptr: *const Block) -> *mut Block {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
// [[nodiscard]] region* mc_schem_clone_region(const region*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_region(ptr: *const Region) -> *mut Region {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
// [[nodiscard]] entity* mc_schem_clone_entity(const entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_entity(ptr: *const Entity) -> *mut Entity {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
// [[nodiscard]] block_entity* mc_schem_clone_block_entity(const block_entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_block_entity(ptr: *const BlockEntity) -> *mut BlockEntity {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
// [[nodiscard]] pending_tick* mc_schem_clone_pending_tick(const pending_tick*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_pending_tick(ptr: *const PendingTick) -> *mut PendingTick {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
// [[nodiscard]] meta_data_ir* mc_schem_clone_meta_data_ir(const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_meta_data_ir(ptr: *const MetaDataIR) -> *mut MetaDataIR {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
// [[nodiscard]] schematic* mc_schem_clone_schematic(const schematic*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_schematic(ptr: *const Schematic) -> *mut Schematic {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_clone_nbt_hashmap(
    ptr: *const HashMap<String, Value>,
) -> *mut HashMap<String, Value> {
    Box::into_raw(Box::from(ptr.as_ref_unchecked().clone()))
}
