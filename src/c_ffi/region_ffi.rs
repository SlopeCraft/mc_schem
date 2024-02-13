use std::collections::HashMap;
use std::ptr::drop_in_place;
use fastnbt::Value;
use crate::c_ffi::{CMapRef, CPendingTickType, CPosDouble, CPosInt, CStringView};
use crate::region::{BlockEntity, Entity, PendingTick, PendingTickInfo};

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