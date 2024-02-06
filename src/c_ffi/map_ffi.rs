use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use std::ptr::null_mut;
use fastnbt::Value;
use crate::c_ffi::{CMapBox, CMapIterator, CMapKeyType, CMapKeyWrapper, CMapRef, CMapValueType, CMapValueWrapper, CStringView};
use crate::region::{BlockEntity, PendingTick};

impl CMapRef {
    pub fn key_value_type(&self) -> (CMapKeyType, CMapValueType) {
        return match self {
            CMapRef::StrStr(_) => (CMapKeyType::String, CMapValueType::String),
            CMapRef::StrValue(_) => (CMapKeyType::String, CMapValueType::NBT),
            CMapRef::PosBlockEntity(_) => (CMapKeyType::Pos, CMapValueType::BlockEntity),
            CMapRef::PosPendingTick(_) => (CMapKeyType::Pos, CMapValueType::PendingTick),
        }
    }
}

impl CMapBox {
    pub fn to_c_map_ref(&self) -> CMapRef {

        return match self {
            CMapBox::StrStr(ss)
            => {
                type V = BTreeMap<String, String>;
                CMapRef::StrStr(ss.as_ref() as *const V as *mut V)
            },
            CMapBox::StrValue(sv)
            => {
                type V = HashMap<String, Value>;
                CMapRef::StrValue(sv.as_ref() as *const V as *mut V)
            },
            CMapBox::PosBlockEntity(pb)
            => {
                type V = HashMap<[i32; 3], BlockEntity>;
                CMapRef::PosBlockEntity(pb.as_ref() as *const V as *mut V)
            },
            CMapBox::PosPendingTick(pp)
            => {
                type V = HashMap<[i32; 3], PendingTick>;
                CMapRef::PosPendingTick(pp.as_ref() as *const V as *mut V)
            },
            CMapBox::None => panic!("Trying to convert CMapBox::None to CMapRef"),
        }
    }
}


#[no_mangle]
extern "C" fn MC_SCHEM_map_unwrap_box(src: *const CMapBox) -> CMapRef {
    unsafe {
        //let src = src as *mut CMapBox;
        let src = &*src;
        return src.to_c_map_ref();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_get_key_type(src: *const CMapRef) -> CMapKeyType {
    unsafe {
        let src = &*src;
        return src.key_value_type().0;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_get_value_type(src: *const CMapRef) -> CMapValueType {
    unsafe {
        let src = &*src;
        return src.key_value_type().1;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_create_map(key_t: CMapKeyType, val_t: CMapValueType, success: *mut bool) -> CMapBox {
    unsafe {
        if key_t == CMapKeyType::String && val_t == CMapValueType::String {
            *success = true;
            return CMapBox::StrStr(Box::new(BTreeMap::new()));
        }
        if key_t == CMapKeyType::String && val_t == CMapValueType::NBT {
            *success = true;
            return CMapBox::StrValue(Box::new(HashMap::new()));
        }
        if key_t == CMapKeyType::Pos && val_t == CMapValueType::BlockEntity {
            *success = true;
            return CMapBox::PosBlockEntity(Box::new(HashMap::new()));
        }
        if key_t == CMapKeyType::Pos && val_t == CMapValueType::PendingTick {
            *success = true;
            return CMapBox::PosPendingTick(Box::new(HashMap::new()));
        }

        *success = false;
        return CMapBox::None;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_release_map(map_box: *mut CMapBox) {
    unsafe {
        *map_box = CMapBox::None;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_find(map: *const CMapRef, key_t: CMapKeyType, val_t: CMapValueType,
                                key: CMapKeyWrapper, ok: *mut bool) -> CMapValueWrapper {
    unsafe {
        let map = map as *mut CMapRef;
        let map = &mut *map;
        if (key_t, val_t) != map.key_value_type() {
            *ok = false;
            return CMapValueWrapper { string: null_mut() };
        }
        *ok = true;
        return match map {
            CMapRef::StrStr(map) => {
                debug_assert!(key_t == CMapKeyType::String);
                debug_assert!(val_t == CMapValueType::String);
                let map = &mut *(*map);
                let s_ptr = match map.get_mut(key.string.to_str()) {
                    Some(v) => v as *mut String,
                    None => null_mut(),
                };
                CMapValueWrapper { string: s_ptr }
            }
            CMapRef::StrValue(map) => {
                debug_assert!(key_t == CMapKeyType::String);
                debug_assert!(val_t == CMapValueType::NBT);
                let map = &mut *(*map);
                let ptr = match map.get_mut(key.string.to_str()) {
                    Some(v) => v as *mut Value,
                    None => null_mut(),
                };
                CMapValueWrapper { nbt: ptr }
            }
            CMapRef::PosBlockEntity(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::BlockEntity);
                let map = &mut *(*map);
                let ptr = match map.get_mut(&key.pos) {
                    Some(v) => v as *mut BlockEntity,
                    None => null_mut(),
                };
                CMapValueWrapper { block_entity: ptr }
            }
            CMapRef::PosPendingTick(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::PendingTick);
                let map = &mut *(*map);
                let ptr = match map.get_mut(&key.pos) {
                    Some(v) => v as *mut PendingTick,
                    None => null_mut(),
                };
                CMapValueWrapper { pending_tick: ptr }
            }
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_length(map: *const CMapRef) -> usize {
    unsafe {
        let map = &*map;
        return match map {
            CMapRef::StrStr(map) => (&**map).len(),
            CMapRef::StrValue(map) => (&**map).len(),
            CMapRef::PosBlockEntity(map) => (&**map).len(),
            CMapRef::PosPendingTick(map) => (&**map).len(),
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_reserve(map: *mut CMapRef, new_capacity: usize) {
    unsafe {
        let map = &*map;
        match map {
            CMapRef::StrStr(_) => {},
            CMapRef::StrValue(map) => {
                let map = &mut **map;
                map.reserve(max(new_capacity, map.len()) - map.len());
            },
            CMapRef::PosBlockEntity(map) => {
                let map = &mut **map;
                map.reserve(max(new_capacity, map.len()) - map.len());
            },
            CMapRef::PosPendingTick(map) => {
                let map = &mut **map;
                map.reserve(max(new_capacity, map.len()) - map.len());
            },
        }
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn MC_SCHEM_map_iterator_first(
    map: *const CMapRef, key_t: CMapKeyType, val_t: CMapValueType, ok: *mut bool) -> CMapIterator {
    unsafe {
        let map = map as *mut CMapRef;
        let map = &mut *map;
        if (key_t, val_t) != map.key_value_type() {
            *ok = false;
            return CMapIterator::None;
        }
        *ok = true;
        return match map {
            CMapRef::StrStr(map) => {
                debug_assert!(key_t == CMapKeyType::String);
                debug_assert!(val_t == CMapValueType::String);
                let map = &mut *(*map);
                CMapIterator::StrStr(map.iter_mut())
            }
            CMapRef::StrValue(map) => {
                debug_assert!(key_t == CMapKeyType::String);
                debug_assert!(val_t == CMapValueType::NBT);
                let map = &mut *(*map);
                CMapIterator::StrValue(map.iter_mut())
            }
            CMapRef::PosBlockEntity(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::BlockEntity);
                let map = &mut *(*map);
                CMapIterator::PosBlockEntity(map.iter_mut())
            }
            CMapRef::PosPendingTick(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::PendingTick);
                let map = &mut *(*map);
                CMapIterator::PosPendingTick(map.iter_mut())
            }
        }
    }
}

#[repr(C)]
struct IterAddReturn {
    key: CMapKeyWrapper,
    value: CMapValueWrapper,
    has_value: bool,
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_iterator_next(it: *mut CMapIterator) -> IterAddReturn {
    unsafe {
        let it = &mut *it;
        let mut ret = IterAddReturn {
            key: CMapKeyWrapper { string: CStringView::from("") },
            value: CMapValueWrapper { string: null_mut() },
            has_value: false,
        };
        match it {
            CMapIterator::None => { return ret; }
            CMapIterator::StrStr(it) => {
                if let Some((key, val)) = it.next() {
                    ret.key.string = CStringView::from(key);
                    ret.value.string = val as *mut String;
                    ret.has_value = true;
                }
            }
            CMapIterator::StrValue(it) => {
                if let Some((key, val)) = it.next() {
                    ret.key.string = CStringView::from(key);
                    ret.value.nbt = val as *mut Value;
                    ret.has_value = true;
                }
            }
            CMapIterator::PosBlockEntity(it) => {
                if let Some((key, val)) = it.next() {
                    ret.key.pos = *key;
                    ret.value.block_entity = val as *mut BlockEntity;
                    ret.has_value = true;
                }
            }
            CMapIterator::PosPendingTick(it) => {
                if let Some((key, val)) = it.next() {
                    ret.key.pos = *key;
                    ret.value.pending_tick = val as *mut PendingTick;
                    ret.has_value = true;
                }
            }
        }
        return ret;
    }
}

// #[no_mangle]
// extern "C" fn MC_SCHEM_map_iterator_equal(a: *const CMapIterator, b: *const CMapIterator) -> bool {
//     unsafe {
//         let a = &*a;
//         let b = &*b;
//         return a == b;
//     }
// }

#[no_mangle]
extern "C" fn MC_SCHEM_map_iterator_length(it: *const CMapIterator) -> usize {
    unsafe {
        let it = &*it;
        return match it {
            CMapIterator::None => { 0 }
            CMapIterator::StrStr(it) => { it.len() }
            CMapIterator::StrValue(it) => { it.len() }
            CMapIterator::PosBlockEntity(it) => { it.len() }
            CMapIterator::PosPendingTick(it) => { it.len() }
        }
    }
}
