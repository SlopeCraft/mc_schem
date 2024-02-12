use std::collections::HashMap;
use std::ptr::drop_in_place;
use fastnbt::Value;
use crate::c_ffi::{CMapRef, CPosDouble, CPosInt};
use crate::region::Entity;

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