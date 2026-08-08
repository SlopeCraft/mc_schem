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
use crate::error::Error;
use crate::region::{BlockEntity, HasPalette};
use crate::{Entity, Region};
use std::ffi::{c_char, c_void, CStr};
use std::ptr::{null, null_mut, slice_from_raw_parts};
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

#[no_mangle]
pub unsafe extern "C" fn mc_schem_error_get_message(
    err: *const Error,
    receiver: *const rust_string_receiver,
) {
    if err.is_null() {
        (*receiver).receive("");
        return;
    }
    (*receiver).receive(&(*err).to_string());
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_region(
    size_x: i32,
    size_y: i32,
    size_z: i32,
) -> *mut Region {
    let shape = [size_x, size_y, size_z];
    let box_ptr = Box::from(Region::with_shape(shape));
    Box::into_raw(box_ptr)
}
/// Create region with given palette. If error, error_dest is box of error and returns null;
/// Otherwise error_dest is null.
// [[nodiscard]] region* mc_schem_create_region_with_palette(
// int32_t size_x, int32_t size_y, int32_t size_z,
// const block* const palette[], size_t palette_size, error** error_dest);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_region_with_palette(
    size_x: i32,
    size_y: i32,
    size_z: i32,
    blocks_ptr: *const *const Block,
    palette_size: usize,
    error: *mut *mut Error,
) -> *mut Region {
    if palette_size <= 0 {
        let err = Box::new(Error::PaletteIsEmpty {
            tag_path: "From API".to_string(),
        });
        *error = Box::into_raw(err);
        return null_mut();
    }

    let shape = [size_x, size_y, size_z];
    let mut box_ptr = Box::from(Region::with_shape(shape));
    box_ptr.palette.clear();
    let blocks_ptr = &*slice_from_raw_parts(blocks_ptr, palette_size);
    for ptr in blocks_ptr {
        box_ptr.palette.push((**ptr).clone());
    }
    *error = null_mut();
    Box::into_raw(box_ptr)
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_destroy_region(region: *mut Region) {
    let _ = Box::from_raw(region);
}
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_name(
    region: *const Region,
    receiver: *const rust_string_receiver,
) {
    (*receiver).receive(&(*region).name);
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_offset(
    region: *const Region,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
) {
    let [x, y, z] = (*region).offset;
    *dest_x = x;
    *dest_y = y;
    *dest_z = z;
}

#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_size(
    region: *const Region,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
) {
    let [y, z, x] = (*region).shape_yzx();
    *dest_x = x;
    *dest_y = y;
    *dest_z = z;
}
//size_t mc_schem_region_palette_get_size(const region*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_palette_get_size(region: *const Region) -> usize {
    (*region).palette.len()
}

//const block* mc_schem_region_palette_get_block(const region*, size_t index);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_palette_get_block(
    region: *const Region,
    index: usize,
) -> *const Block {
    if index >= (*region).palette().len() {
        return null();
    }
    (*region).palette.as_ptr().add(index)
}

// Add block into palette (deep copy). If identical block already exist in
// palette, don't copy; otherwise append. Returns index of this block in palette
// uint16_t mc_schem_region_find_or_append_to_palette(region* region, const block* block);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_add_to_palette(
    region: *mut Region,
    new_blk: *const Block,
) -> u16 {
    (*region).find_or_append_to_palette(&*new_blk)
}

//size_t mc_schem_region_get_entities_count(const region*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_entities_count(region: *const Region) -> usize {
    (*region).entities.len()
}
//const entity* mc_schem_region_get_entity(const region*, size_t index);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_entity(
    region: *const Region,
    idx: usize,
) -> *const Entity {
    if idx >= (*region).entities.len() {
        return null();
    }
    &(&(*region).entities)[idx]
}
//entity* mc_schem_region_get_entity_mut(region*, size_t index);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_entity_mut(
    region: *mut Region,
    idx: usize,
) -> *mut Entity {
    if idx >= (*region).entities.len() {
        return null_mut();
    }
    &mut (&mut (*region).entities)[idx]
}
//void mc_schem_region_erase_entity(region*, size_t index);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_erase_entity(region: *mut Region, idx: usize) {
    (*region).entities.remove(idx);
}
/// Clone entity into region, returns index
//size_t mc_schem_region_add_entity(region*, const entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_add_entity(
    region: *mut Region,
    entity_ptr: *const Entity,
) -> usize {
    (*region).entities.push((*entity_ptr).clone());

    (*region).entities.len() - 1
}

// size_t mc_schem_region_get_block_entities_count(const region*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_block_entities_count(region: *const Region) -> usize {
    (*region).block_entities.len()
}
// void mc_schem_region_visit_block_entities(const region*, void (*callback)(int32_t x, int32_t y, const block_entity*, void*), void* custom_data);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_visit_block_entities(
    region: *const Region,
    callback: extern "C" fn(
        x: i32,
        y: i32,
        z: i32,
        be: *const BlockEntity,
        custom_data: *mut c_void,
    ),
    custom_data: *mut c_void,
) {
    for (pos, be) in &(*region).block_entities {
        let [x, y, z] = *pos;
        callback(x, y, z, be, custom_data);
    }
}
// const block_entity* mc_schem_region_get_block_entity(const region*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_block_entity(
    region: *const Region,
    x: i32,
    y: i32,
    z: i32,
) -> *const BlockEntity {
    let be = (*region).block_entities.get(&[x, y, z]);

    if let Some(ret) = be {
        return ret;
    }
    null()
}
// block_entity* mc_schem_region_get_block_entity_mut(region*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_block_entity_mut(
    region: *mut Region,
    x: i32,
    y: i32,
    z: i32,
) -> *mut BlockEntity {
    let be = (*region).block_entities.get_mut(&[x, y, z]);

    if let Some(ret) = be {
        return ret;
    }
    null_mut()
}

/// Copy and insert block entity into given coordinate. If previous BE exists, it
/// will be moved out and boxed and returned as ptr. If be is null, erase old value
// block_entity* mc_schem_region_add_block_entity(region*, int32_t x, int32_t y, int32_t z, const block_entity*nullable);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_add_block_entity(
    region: *mut Region,
    x: i32,
    y: i32,
    z: i32,
    new_be_nullable: *const BlockEntity,
) -> *mut BlockEntity {
    let old;
    if new_be_nullable.is_null() {
        old = (*region).block_entities.remove(&[x, y, z]);
    } else {
        old = (*region)
            .block_entities
            .insert([x, y, z], (*new_be_nullable).clone());
    }

    if let Some(old) = old {
        let ret = Box::new(old);
        return Box::into_raw(ret);
    }
    null_mut()
}
