use std::collections::HashMap;
use std::mem::swap;
use std::ptr::{drop_in_place, null, null_mut};
use fastnbt::Value;
use crate::Block;
use crate::c_ffi::{CMapRef, CNumberId, CPendingTickType, CPosDouble, CPosInt, CRegionBlockInfo, CStringView, error_to_box};
use crate::region::{BlockEntity, Entity, PendingTick, PendingTickInfo, Region};
use crate::error::Error;

#[no_mangle]
extern "C" fn MC_SCHEM_create_entity() -> Box<Entity> {
    return Box::new(Entity::new());
}

#[no_mangle]
extern "C" fn MC_SCHEM_release_entity(entity_box: *mut Box<Entity>) {
    unsafe {
        drop_in_place(entity_box);
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_entity(a: *mut Entity, b: *mut Entity) {
    swap(&mut *a, &mut *b);
}

#[no_mangle]
extern "C" fn MC_SCHEM_entity_get_block_pos(entity: *const Entity) -> CPosInt {
    unsafe {
        let entity = &*entity;
        return CPosInt { pos: entity.block_pos };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_entity_get_pos(entity: *const Entity) -> CPosDouble {
    unsafe {
        let entity = &*entity;
        return CPosDouble { pos: entity.position };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_entity_set_block_pos(entity: *mut Entity, pos: CPosInt) {
    unsafe {
        (*entity).block_pos = pos.pos;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_entity_set_pos(entity: *mut Entity, pos: CPosDouble) {
    unsafe {
        (*entity).position = pos.pos;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_entity_get_tags(entity: *const Entity) -> CMapRef {
    unsafe {
        let entity = &*entity;
        return CMapRef::StrValue(&entity.tags
            as *const HashMap<String, Value>
            as *mut HashMap<String, Value>);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_create_block_entity() -> Box<BlockEntity> {
    return Box::new(BlockEntity::new());
}

#[no_mangle]
extern "C" fn MC_SCHEM_release_block_entity(b: *mut Box<BlockEntity>) {
    unsafe {
        drop_in_place(b);
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_block_entity(a: *mut BlockEntity, b: *mut BlockEntity) {
    swap(&mut *a, &mut *b);
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_entity_get_tags(be: *const BlockEntity) -> CMapRef {
    unsafe {
        let be = &*be;
        let map = &be.tags as *const HashMap<String, Value> as *mut HashMap<String, Value>;
        return CMapRef::StrValue(map);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_create_pending_tick() -> Box<PendingTick> {
    return Box::new(PendingTick { priority: 0, sub_tick: 0, time: 0, info: PendingTickInfo::default() });
}

#[no_mangle]
extern "C" fn MC_SCHEM_release_pending_tick(b: *mut Box<PendingTick>) {
    unsafe {
        drop_in_place(b);
    }
}


#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_pending_tick(a: *mut PendingTick, b: *mut PendingTick) {
    swap(&mut *a, &mut *b);
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_get_priority(p: *const PendingTick) -> i32 {
    unsafe {
        return (*p).priority;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_set_priority(p: *mut PendingTick, priority: i32) {
    unsafe {
        (*p).priority = priority;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_get_sub_tick(p: *const PendingTick) -> i64 {
    unsafe {
        return (*p).sub_tick;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_set_sub_tick(p: *mut PendingTick, sub_tick: i64) {
    unsafe {
        (*p).sub_tick = sub_tick;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_get_time(p: *const PendingTick) -> i32 {
    unsafe {
        return (*p).time;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_set_time(p: *mut PendingTick, time: i32) {
    unsafe {
        (*p).time = time;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_get_id(p: *const PendingTick) -> CStringView {
    unsafe {
        return match &(&*p).info {
            PendingTickInfo::Fluid { id } => CStringView::from(&id),
            PendingTickInfo::Block { id } => CStringView::from(&id),
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_get_type(p: *const PendingTick) -> CPendingTickType {
    unsafe {
        return match &(&*p).info {
            PendingTickInfo::Fluid { .. } => CPendingTickType::Fluid,
            PendingTickInfo::Block { .. } => CPendingTickType::Block,
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_pending_tick_set_info(p: *mut PendingTick, t: CPendingTickType, id: CStringView) {
    unsafe {
        let p = &mut *p;
        match t {
            CPendingTickType::Fluid => p.info = PendingTickInfo::Fluid { id: id.to_string() },
            CPendingTickType::Block => p.info = PendingTickInfo::Block { id: id.to_string() },
        }
    }
}


#[no_mangle]
extern "C" fn MC_SCHEM_create_region() -> Box<Region> {
    return Box::new(Region::new());
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_release_region(b: *mut Box<Region>) {
    drop_in_place(b);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_region(a: *mut Region, b: *mut Region) {
    swap(&mut *a, &mut *b);
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_name(region: *const Region) -> CStringView {
    return CStringView::from((*region).name.as_str());
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_set_name(region: *mut Region, name: CStringView) {
    (*region).name = name.to_string();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_offset(region: *const Region) -> CPosInt {
    return CPosInt { pos: (*region).offset };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_set_offset(region: *mut Region, offset: CPosInt) {
    (*region).offset = offset.pos;
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_palette(region: *const Region, len: *mut usize) -> *mut Block {
    let region = &mut *(region as *mut Region);
    *len = region.palette.len();
    return region.palette.as_mut_ptr();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_set_palette(region: *mut Region, pal: *const *const Block, len: usize) {
    let region = &mut *(region);
    let mut new_pal = Vec::with_capacity(len);
    for idx in 0..len {
        let p = *(pal.clone().add(idx));
        new_pal.push((*p).clone());
    }
    region.palette = new_pal;
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block_entities(region: *const Region) -> CMapRef {
    let region = &mut *(region as *mut Region);
    return CMapRef::PosBlockEntity(&mut region.block_entities as *mut HashMap<[i32; 3], BlockEntity>);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_pending_ticks(region: *const Region) -> CMapRef {
    let region = &mut *(region as *mut Region);
    return CMapRef::PosPendingTick(&mut region.pending_ticks as *mut HashMap<[i32; 3], PendingTick>);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_entities(region: *const Region, len: *mut usize) -> *mut Entity {
    let region = &mut *(region as *mut Region);
    *len = region.entities.len();
    return region.entities.as_mut_ptr();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block_index_array(region: *const Region) -> *mut u16 {
    let region = &mut *(region as *mut Region);
    return region.array.as_mut_ptr();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_number_id_array(region: *const Region) -> *mut CNumberId {
    let region = &mut *(region as *mut Region);
    if let Some(arr) = &mut region.array_number_id_damage {
        return arr.as_mut_ptr() as *mut CNumberId;
    }
    return null_mut();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_shape(region: *const Region) -> CPosInt {
    return CPosInt { pos: (*region).shape() };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_reshape(region: *mut Region, new_size: CPosInt) {
    (*region).reshape(&new_size.pos);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block(region: *mut Region, r_pos: CPosInt) -> *const Block {
    if let Some(blk) = (*region).block_at(r_pos.pos) {
        return blk as *const Block;
    }
    return null();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_set_block(region: *mut Region, r_pos: CPosInt, block: *const Block) -> bool {
    let result = (*region).set_block(r_pos.pos, &*block);
    return result.is_ok();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block_index(region: *const Region, r_pos: CPosInt) -> u16 {
    return (*region).block_index_at(r_pos.pos).unwrap();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_set_block_index(region: *mut Region, r_pos: CPosInt, new_idx: u16) -> bool {
    return (*region).set_block_id(r_pos.pos, new_idx).is_ok();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_volume(region: *const Region) -> u64 {
    return (*region).volume();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_total_blocks(region: *const Region, include_air: bool) -> u64 {
    return (*region).total_blocks(include_air);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block_index_of_air(region: *const Region, ok: *mut bool) -> u16 {
    if let Some(id) = (*region).block_index_of_air() {
        *ok = true;
        return id;
    }
    *ok = false;
    return u16::MAX;
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block_index_of_structure_void(region: *const Region, ok: *mut bool) -> u16 {
    if let Some(id) = (*region).block_index_of_structure_void() {
        *ok = true;
        return id;
    }
    *ok = false;
    return u16::MAX;
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_contains_coordinate(region: *const Region, r_pos: CPosInt) -> bool {
    return (*region).contains_coord(r_pos.pos);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_get_block_info(region: *const Region, r_pos: CPosInt) -> CRegionBlockInfo {
    let region = &*region;
    let mut result = CRegionBlockInfo::default();
    result.block_index = region.block_index_at(r_pos.pos).unwrap();
    result.block_entity = if let Some(be) = region.block_entity_at(r_pos.pos) {
        be as *const BlockEntity as *mut BlockEntity
    } else {
        null_mut()
    };
    result.pending_tick = if let Some(pt) = region.pending_tick_at(r_pos.pos) {
        pt as *const PendingTick as *mut PendingTick
    } else {
        null_mut()
    };

    return result;
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_region_shrink_palette(region: *mut Region) -> Option<Box<Error>> {
    let err = (*region).shrink_palette();
    if let Err(err) = err {
        return error_to_box(Some(err));
    }
    return None;
}