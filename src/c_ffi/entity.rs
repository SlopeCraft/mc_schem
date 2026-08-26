use std::collections::HashMap;
use std::ffi::c_double;
use fastnbt::Value;
use crate::region::Entity;

// void mc_schem_entity_get_position(const entity *, int32_t* x, int32_t* y,int32_t* z, double* fp_x, double* fp_y,double* fp_z)
#[no_mangle]
pub unsafe extern "C" fn mc_schem_entity_get_position(
    entity: *const Entity,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
    dest_fp_x: *mut c_double,
    dest_fp_y: *mut c_double,
    dest_fp_z: *mut c_double,
) {
    {
        let [x, y, z] = (*entity).block_pos;
        *dest_x = x;
        *dest_y = y;
        *dest_z = z;
    }
    {
        let [x, y, z] = (*entity).position;
        *dest_fp_x = x as c_double;
        *dest_fp_y = y as c_double;
        *dest_fp_z = z as c_double;
    }
}

// const nbt_hashmap* mc_schem_entity_get_tags(const entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_entity_get_tags(entity: *const Entity) -> *const HashMap<String, Value> {
    &entity.as_ref_unchecked().tags
}
// nbt_hashmap* mc_schem_entity_get_tags_mut(entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_entity_get_tags_mut(entity: *mut Entity) -> *mut HashMap<String, Value> {
    &mut (entity.as_mut_unchecked().tags)
}
/// Deep copy new_value into entity, returns old value by box (transfer ownership
// nbt_hashmap* mc_schem_entity_set_tags(entity*, const nbt_hashmap* new_value);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_entity_set_tags(entity: *mut Entity, new_value: *const HashMap<String, Value>) -> *mut HashMap<String, Value> {
    let mut new = new_value.as_ref_unchecked().clone();
    std::mem::swap(&mut entity.as_mut_unchecked().tags, &mut new);
    // After swap, variable new contains old value
    let b = Box::from(new);
    Box::into_raw(b)
}