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
mod error;
mod region;

use crate::block::{Block};
use crate::error::Error;
use crate::region::{BlockEntity, HasPalette};
use crate::{Entity, Region};
use std::ffi::{c_void};
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
