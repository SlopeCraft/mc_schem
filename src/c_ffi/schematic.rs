// [[nodiscard]] schematic* mc_schem_create_schematic();

use std::ptr::{null, null_mut};
use crate::block::Block;
use crate::{BlockEntity, PendingTick, Region};
use crate::schem::{MetaDataIR, Schematic};

#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_schematic() -> *mut Schematic {
    Box::into_raw(Box::new(Schematic::new()))
}


// [[nodiscard]] const meta_data_ir* mc_schem_schematic_get_meta_data(const schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_meta_data(schem: *const Schematic) -> *const MetaDataIR {
    &(schem.as_ref_unchecked().metadata)
}
// [[nodiscard]] meta_data_ir* mc_schem_schematic_get_meta_data_mut(schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_meta_data_mut(schem: *mut Schematic) -> *mut MetaDataIR {
    &mut (schem.as_mut_unchecked().metadata)
}
// /// Deep copy given metadata, move old value to heap and return
// meta_data_ir* mc_schem_schematic_set_meta_data(schematic*, const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_set_meta_data(schem: *mut Schematic, new_ir: *const MetaDataIR) -> *mut MetaDataIR {
    let mut val = new_ir.as_ref_unchecked().clone();
    std::mem::swap(&mut val, &mut (schem.as_mut_unchecked().metadata));
    Box::into_raw(Box::from(val))
}
// [[nodiscard]] size_t mc_schem_schematic_get_regions_count(const schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_regions_count(schem: *const Schematic) -> usize {
    schem.as_ref_unchecked().regions.len()
}

// [[nodiscard]] const region* mc_schem_schematic_get_region(const schematic*,size_t idx);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_region(schem: *const Schematic, idx: usize) -> *const Region {
    if let Some(r) = schem.as_ref_unchecked().regions.get(idx) {
        return r;
    }
    null()
}
// [[nodiscard]] region* mc_schem_schematic_get_region_mut(schematic*,size_t idx);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_region_mut(schem: *mut Schematic, idx: usize) -> *mut Region {
    if let Some(r) = schem.as_mut_unchecked().regions.get_mut(idx) {
        return r;
    }
    null_mut()
}
// /// Remove a region from schematic, move to heap and return its pointer.
// /// Transfer ownership.
// [[nodiscard]] region* mc_schem_schematic_remove_region(schematic*,size_t idx);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_remove_region(schem: *mut Schematic, idx: usize) -> *mut Region {
    let regions = &mut (schem.as_mut_unchecked().regions);
    if idx >= regions.len() {
        return null_mut();
    }
    let removed = regions.remove(idx);
    Box::into_raw(Box::from(removed))

}
// /// Remove all regions from schematic
// void mc_schem_schematic_clear_all_regions(schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_clear_all_regions(schem: *mut Schematic) {
    schem.as_mut_unchecked().regions.clear()
}
// /// Deep copy new_region into given index, return its pointer in schematic. If
// /// index out of range (for example, insert to index=4 with only 3 regions,
// /// nothing will be done and returns nullptr)
// region* mc_schem_schematic_insert_region(schematic*, const region* new_region, size_t index);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_insert_region(schem: *mut Schematic, region: *const Region, idx: usize) -> *mut Region {
    if idx > schem.as_mut_unchecked().regions.len() {
        return null_mut();
    }
    schem.as_mut_unchecked().regions.insert_mut(idx, region.as_ref_unchecked().clone())
}

/// Returns positive value if coordinate hits a region. Otherwise return -1. All
/// negative value should be considered as invalid
/// The word "first" means first hit region. Schematic have multiple regions,
/// usually no overlap is expected. If multiple regions overlaps, blocks in
/// first region will live
// [[nodiscard]] ptrdiff_t mc_schem_schematic_get_first_region_index_at(const schematic*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_first_region_index_at(schem: *const Schematic, x: i32, y: i32, z: i32) -> isize {
    if let Some(idx) = schem.as_ref_unchecked().first_region_index_at([x, y, z]) {
        return idx as isize;
    }
    -1
}
// [[nodiscard]] uint16_t mc_schem_schematic_get_first_block_index_at(const schematic*, int32_t x, int32_t y, int32_t z, bool* ok_nonnull);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_first_block_index_at(schem: *const Schematic, x: i32, y: i32, z: i32, ok_nonnull: *mut bool) -> u16 {
    if let Some(idx) = schem.as_ref_unchecked().first_block_index_at([x, y, z]) {
        *ok_nonnull = true;
        return idx;
    }
    *ok_nonnull = false;
    u16::MAX
}
// [[nodiscard]] const block* mc_schem_schematic_get_first_block_at( const schematic*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_first_block_at(schem: *const Schematic, x: i32, y: i32, z: i32) -> *const Block {
    if let Some(blk) = schem.as_ref_unchecked().first_block_at([x, y, z]) {
        return blk;
    }
    null()
}
// [[nodiscard]] const block_entity* mc_schem_schematic_get_first_block_entity_at(const schematic*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_first_block_entity_at(schem: *const Schematic, x: i32, y: i32, z: i32) -> *const BlockEntity {
    if let Some(be) = schem.as_ref_unchecked().first_block_entity_at([x, y, z]) {
        return be;
    }
    null()
}
// [[nodiscard]] size_t mc_schem_schematic_get_first_pending_ticks_count_at(const schematic*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_first_pending_ticks_count_at(schem: *const Schematic, x: i32, y: i32, z: i32) -> usize {
    schem.as_ref_unchecked().first_pending_tick_at([x, y, z]).len()
}
// [[nodiscard]] const pending_tick* mc_schem_schematic_get_first_pending_ticks_at(const schematic*, int32_t x, int32_t y, int32_t z,size_t pending_tick_index);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_first_pending_ticks_at(schem: *const Schematic, x: i32, y: i32, z: i32, pending_tick_index: usize) -> *const PendingTick {
    let pts = schem.as_ref_unchecked().first_pending_tick_at([x, y, z]);
    if let Some(pt) = pts.get(pending_tick_index) {
        return pt;
    }
    null()
}
// void mc_schem_schematic_get_shape(const schematic*, int32_t* x, int32_t* y,int32_t* z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_shape(schem: *const Schematic, x: *mut i32, y: *mut i32, z: *mut i32) {
    let shape = schem.as_ref_unchecked().shape();
    *x = shape[0];
    *y = shape[1];
    *z = shape[2];
}
// [[nodiscard]] uint64_t mc_schem_schematic_get_volume(const schematic*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_volume(schem: *const Schematic) -> u64 {
    schem.as_ref_unchecked().volume()
}
// [[nodiscard]] uint64_t mc_schem_schematic_get_total_blocks(const schematic*,bool include_dir);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_total_blocks(schem: *const Schematic, include_air: bool) -> u64 {
    schem.as_ref_unchecked().total_blocks(include_air)
}
// /// Merge all regions without changing original schematic
// [[nodiscard]] region* mc_schem_schematic_to_single_region(const schematic*, const block* background_block);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_to_single_region(schem: *const Schematic, background_blk: *const Block) -> *mut Region {
    let new_reg = schem.as_ref_unchecked().to_single_region(background_blk.as_ref_unchecked());
    Box::into_raw(Box::new(new_reg))
}
// /// Merge all regions in place
// void mc_schem_schematic_merge_regions(schematic*,const block* background_block);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_merge_regions(schem: *mut Schematic, background_blk: *const Block) {
    schem.as_mut_unchecked().merge_regions(background_blk.as_ref_unchecked());
}