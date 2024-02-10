use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use std::ptr::null_mut;
use fastnbt::Value;
use crate::c_ffi::{CMapBox, CMapIterator, CMapKeyType, CMapKeyWrapper, CMapRef, CMapValueType, CMapValueWrapper, CStringView, KVRef};
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
extern "C" fn MC_SCHEM_map_capacity(map: *const CMapRef) -> usize {
    unsafe {
        let map = &*map;
        return match map {
            CMapRef::StrStr(map) => (&**map).len(),
            CMapRef::StrValue(map) => (&**map).capacity(),
            CMapRef::PosBlockEntity(map) => (&**map).capacity(),
            CMapRef::PosPendingTick(map) => (&**map).capacity(),
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
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::StrStr { iter: map.iter_mut(), deref }
            }
            CMapRef::StrValue(map) => {
                debug_assert!(key_t == CMapKeyType::String);
                debug_assert!(val_t == CMapValueType::NBT);
                let map = &mut *(*map);
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::StrValue { iter: map.iter_mut(), deref }
            }
            CMapRef::PosBlockEntity(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::BlockEntity);
                let map = &mut *(*map);
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::PosBlockEntity { iter: map.iter_mut(), deref }
            }
            CMapRef::PosPendingTick(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::PendingTick);
                let map = &mut *(*map);
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::PosPendingTick { iter: map.iter_mut(), deref }
            }
        }
    }
}

#[repr(C)]
struct IterDerefResult {
    key: CMapKeyWrapper,
    value: CMapValueWrapper,
    has_value: bool,
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_iterator_add(it: *mut CMapIterator) {
    unsafe {
        let it = &mut *it;
        match it {
            CMapIterator::None => {},
            CMapIterator::StrStr { iter, deref } => *deref = KVRef::new(iter.next()),
            CMapIterator::StrValue { iter, deref } => *deref = KVRef::new(iter.next()),
            CMapIterator::PosBlockEntity { iter, deref } => *deref = KVRef::new(iter.next()),
            CMapIterator::PosPendingTick { iter, deref } => *deref = KVRef::new(iter.next()),
        }
        return;
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_iterator_deref(it: *const CMapIterator) -> IterDerefResult {
    unsafe {
        let it = &*it;
        let mut ret = IterDerefResult {
            key: CMapKeyWrapper { string: CStringView::from("") },
            value: CMapValueWrapper { string: null_mut() },
            has_value: false,
        };
        match it {
            CMapIterator::None => { return ret; }
            CMapIterator::StrStr { iter: _, deref } => {
                if !deref.is_null() {
                    ret.key.string = CStringView::from(&*deref.key);
                    ret.value.string = deref.value;
                    ret.has_value = true;
                }
            }
            CMapIterator::StrValue { iter: _, deref } => {
                if !deref.is_null() {
                    ret.key.string = CStringView::from(&*deref.key);
                    ret.value.nbt = deref.value;
                    ret.has_value = true;
                }
            }
            CMapIterator::PosBlockEntity { iter: _, deref } => {
                if !deref.is_null() {
                    ret.key.pos = *deref.key;
                    ret.value.block_entity = deref.value;
                    ret.has_value = true;
                }
            }
            CMapIterator::PosPendingTick { iter: _, deref } => {
                if !deref.is_null() {
                    ret.key.pos = *deref.key;
                    ret.value.pending_tick = deref.value;
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
            CMapIterator::StrStr { iter, .. } => { iter.len() }
            CMapIterator::StrValue { iter, .. } => { iter.len() }
            CMapIterator::PosBlockEntity { iter, .. } => { iter.len() }
            CMapIterator::PosPendingTick { iter, .. } => { iter.len() }
        }
    }
}
