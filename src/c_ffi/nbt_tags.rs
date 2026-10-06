use std::collections::HashMap;
use fastnbt::Value;

//nbt_hashmap* mc_schem_create_nbt_hashmap()
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap() -> *mut HashMap<String, Value> {
    Box::into_raw(Box::from(HashMap::new()))
}