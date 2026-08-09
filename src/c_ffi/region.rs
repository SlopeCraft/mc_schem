use crate::block::Block;
use crate::c_ffi::rust_string_receiver;
use crate::error::Error;
use crate::region::{BlockEntity, HasPalette};
use crate::{Entity, PendingTick, Region};
use std::ffi::c_void;
use std::ptr::{null, null_mut, slice_from_raw_parts};

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
//entity* mc_schem_region_erase_entity(region*, size_t index);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_erase_entity(
    region: *mut Region,
    idx: usize,
) -> *mut Entity {
    let old = (*region).entities.remove(idx);
    let ret = Box::from(old);
    Box::into_raw(ret)
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

// size_t mc_schem_region_get_pending_ticks_count(const region*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_region_get_pending_ticks_count(
    region: *const Region,
    x: i32,
    y: i32,
    z: i32,
) -> usize {
    (*region)
        .pending_ticks
        .get(&[x, y, z])
        .unwrap_or(&vec![])
        .len()
}
// const pending_tick* mc_schem_region_get_pending_tick(const region*, int32_t x, int32_t y, int32_t z, size_t idx);
pub unsafe extern "C" fn mc_schem_region_get_pending_tick(
    region: *const Region,
    x: i32,
    y: i32,
    z: i32,
    idx: usize,
) -> *const PendingTick {
    let opt = (*region).pending_ticks.get(&[x, y, z]);
    if let Some(val) = opt {
        if idx < val.len() {
            return &val[idx];
        }
    }
    null()
}
// pending_tick* mc_schem_region_get_pending_tick_mut(region*, int32_t x, int32_t y, int32_t z, size_t idx);
pub unsafe extern "C" fn mc_schem_region_get_pending_tick_mut(
    region: *mut Region,
    x: i32,
    y: i32,
    z: i32,
    idx: usize,
) -> *mut PendingTick {
    let opt = (*region).pending_ticks.get_mut(&[x, y, z]);
    if let Some(val) = opt {
        if idx < val.len() {
            return &mut val[idx];
        }
    }
    null_mut()
}