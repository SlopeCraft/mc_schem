/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2026 ToKiNoBug

 This program is free software: you can redistribute it and/or modify it under
 the terms of the GNU General Public License as published by the Free Software
 Foundation, either version 3 of the License, or (at your option) any later
 version.

 This program is distributed in the hope that it will be useful, but WITHOUT ANY
 WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 PARTICULAR PURPOSE.  See the GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use crate::BlockEntity;
use fastnbt::Value;
use std::collections::HashMap;

// block_entity* mc_schem_create_block_entity();
#[no_mangle]
pub extern "C" fn mc_schem_create_block_entity() -> *mut BlockEntity {
    Box::into_raw(Box::new(BlockEntity::new()))
}
// const nbt_hashmap* mc_schem_block_entity_get_tags(const block_entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_entity_get_tags(
    be: *const BlockEntity,
) -> *const HashMap<String, Value> {
    &(be.as_ref_unchecked().tags)
}
// nbt_hashmap* mc_schem_block_entity_get_tags_mut(block_entity*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_entity_get_tags_mut(
    be: *mut BlockEntity,
) -> *mut HashMap<String, Value> {
    &mut (be.as_mut_unchecked().tags)
}
// /// Deep copy new_value into entity, returns old value by box (transfer
// /// ownership
// nbt_hashmap* mc_schem_block_entity_set_tags(block_entity*,const nbt_hashmap* new_value);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_block_entity_set_tags(
    be: *mut BlockEntity,
    new_value: *const HashMap<String, Value>,
) -> *mut HashMap<String, Value> {
    assert!(!new_value.is_null());
    // Prevent self-copy
    assert_ne!(new_value, &(be.as_ref_unchecked().tags));
    let mut ret = new_value.as_ref_unchecked().clone();
    std::mem::swap(&mut ret, &mut (be.as_mut_unchecked().tags));
    Box::into_raw(Box::new(ret))
}
