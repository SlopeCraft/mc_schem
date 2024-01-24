use std::ffi::c_void;
use std::ptr::{null, null_mut};
use fastnbt::Value;
use crate::c_ffi::{CBlockEntity, CMapRef, CMapRefKeyType, CMapRefValueType, CNBTValue, CPendingTick, SchemString};
use crate::region::{BlockEntity, PendingTick};

impl CMapRef {
    pub fn key_type(&self) -> CMapRefKeyType {
        return match self {
            CMapRef::StrStr(_) => CMapRefKeyType::String,
            CMapRef::StrValue(_) => CMapRefKeyType::String,
            CMapRef::PosBlockEntity(_) => CMapRefKeyType::Pos,
            CMapRef::PosPendingTick(_) => CMapRefKeyType::Pos,
        };
    }

    pub fn value_type(&self) -> CMapRefValueType {
        return match self {
            CMapRef::StrStr(_) => CMapRefValueType::String,
            CMapRef::StrValue(_) => CMapRefValueType::NBT,
            CMapRef::PosBlockEntity(_) => CMapRefValueType::BlockEntity,
            CMapRef::PosPendingTick(_) => CMapRefValueType::PendingTick,
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_get_key_type(map: *const CMapRef) -> CMapRefKeyType {
    unsafe {
        return (*map).key_type();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_get_value_type(map: *const CMapRef) -> CMapRefValueType {
    unsafe {
        return (*map).value_type();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_get_size(map: *const CMapRef) -> usize {
    unsafe {
        return match *map {
            CMapRef::StrStr(m) => (*m).len(),
            CMapRef::StrValue(m) => (*m).len(),
            CMapRef::PosBlockEntity(m) => (*m).len(),
            CMapRef::PosPendingTick(m) => (*m).len(),
        };
    }
}


#[no_mangle]
extern "C" fn MC_SCHEM_map_find_const(map: *const CMapRef, key_type: CMapRefKeyType, key: *const c_void, value: *mut c_void) -> bool {
    unsafe {
        if (*map).key_type() != key_type {
            return false;
        }

        if key_type == CMapRefKeyType::String {
            let key: &SchemString = &*(key as *const SchemString);
            let key = key.to_str();

            return match *map {
                CMapRef::StrStr(m) => {
                    let find_opt = (*m).get(key);
                    let value = value as *mut SchemString;
                    match find_opt {
                        Some(s) => {
                            *value = SchemString::new(s);
                        }
                        None => { *value = SchemString { begin: null(), end: null() }; }
                    }
                    true
                }
                CMapRef::StrValue(m) => {
                    let find_opt = (*m).get(key);
                    let value = value as *mut CNBTValue;
                    match find_opt {
                        Some(v) => {
                            let v = v as *const Value as *mut Value;
                            *value = CNBTValue::Ref(v);
                        }
                        None => {
                            *value = CNBTValue::Ref(null_mut());
                        }
                    }
                    true
                }
                _ => { false }
            };
        }

        if key_type == CMapRefKeyType::Pos {
            let key = key as *const [i32; 3];
            let key = &*key;
            return match *map {
                CMapRef::PosBlockEntity(m) => {
                    let val = (*m).get(key);
                    let dest = value as *mut CBlockEntity;
                    match val {
                        Some(val) => {
                            let val = val as *const BlockEntity as *mut BlockEntity;
                            *dest = CBlockEntity::Ref(val);
                        }
                        None => {
                            *dest = CBlockEntity::Ref(null_mut());
                        }
                    }
                    true
                }
                CMapRef::PosPendingTick(m) => {
                    let val = (*m).get(key);
                    let dest = value as *mut CPendingTick;
                    match val {
                        Some(val) => {
                            let val = val as *const PendingTick as *mut PendingTick;
                            *dest = CPendingTick::Ref(val);
                        }
                        None => { *dest = CPendingTick::Ref(null_mut()); }
                    }
                    true
                }
                _ => { false }
            };
        }
        return false;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_find_mut(map: *mut CMapRef, key_type: CMapRefKeyType, key: *const c_void, value: *mut c_void) -> bool {
    return MC_SCHEM_map_find_const(map, key_type, key, value);
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_contains_key(map: *const CMapRef, key_type: CMapRefKeyType, key: *const c_void) -> bool {
    unsafe {
        if (*map).key_type() != key_type {
            return false;
        }

        if key_type == CMapRefKeyType::String {
            let key: &SchemString = &*(key as *const SchemString);
            let key = key.to_str();

            return match *map {
                CMapRef::StrStr(m) => {
                    (*m).contains_key(key)
                }
                CMapRef::StrValue(m) => {
                    (*m).contains_key(key)
                }
                _ => { false }
            };
        }

        if key_type == CMapRefKeyType::Pos {
            let key = key as *const [i32; 3];
            let key = &*key;
            return match *map {
                CMapRef::PosBlockEntity(m) => {
                    (*m).contains_key(key)
                }
                CMapRef::PosPendingTick(m) => {
                    (*m).contains_key(key)
                }
                _ => { false }
            };
        }
        return false;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_insert(map: *mut CMapRef, key_type: CMapRefKeyType, key: *const c_void, value: *const c_void) -> bool {
    unsafe {
        if (*map).key_type() != key_type {
            return false;
        }
        if key_type == CMapRefKeyType::String {
            let key: &SchemString = &*(key as *const SchemString);
            let key = key.to_string();

            return match *map {
                CMapRef::StrStr(m) => {
                    let value = value as *const SchemString;
                    let value = (&*value).to_string();
                    (*m).insert(key, value);
                    true
                }
                CMapRef::StrValue(m) => {
                    let value = value as *const CNBTValue;
                    let value = (&*value).get_ref().clone();
                    (*m).insert(key, value);
                    true
                }
                _ => { false }
            };
        }

        if key_type == CMapRefKeyType::Pos {
            let key = key as *const [i32; 3];
            let key = (&*key).clone();
            return match *map {
                CMapRef::PosBlockEntity(m) => {
                    let value = value as *const CBlockEntity;
                    let value = (&*value).get_ref().clone();
                    (*m).insert(key, value);
                    true
                }
                CMapRef::PosPendingTick(m) => {
                    let value = value as *const CPendingTick;
                    let value = (&*value).get_ref().clone();
                    (*m).insert(key, value);
                    true
                }
                _ => { false }
            };
        }

        return false;
    }
}