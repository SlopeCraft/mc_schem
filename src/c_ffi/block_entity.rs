use std::collections::HashMap;
use fastnbt::Value;
use crate::BlockEntity;

// block_entity* mc_schem_create_block_entity();
#[no_mangle]
pub extern "C" fn mc_schem_create_block_entity() -> *mut BlockEntity {
    Box::into_raw(Box::new(BlockEntity::new()))
}
// const nbt_hashmap* mc_schem_block_entity_get_tags(const block_entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_entity_get_tags(be: *const BlockEntity) -> *const HashMap<String, Value> {
    &(be.as_ref_unchecked().tags)
}
// nbt_hashmap* mc_schem_block_entity_get_tags_mut(block_entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_entity_get_tags_mut(be: *mut BlockEntity) -> *mut HashMap<String, Value> {
    &mut (be.as_mut_unchecked().tags)
}
// /// Deep copy new_value into entity, returns old value by box (transfer
// /// ownership
// nbt_hashmap* mc_schem_block_entity_set_tags(block_entity*,const nbt_hashmap* new_value);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_entity_set_tags(be: *mut BlockEntity, new_value: *const HashMap<String, Value>) -> *mut HashMap<String, Value> {
    let mut ret = new_value.as_ref_unchecked().clone();
    std::mem::swap(&mut ret, &mut (be.as_mut_unchecked().tags));
    Box::into_raw(Box::new(ret))
}