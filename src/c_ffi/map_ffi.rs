/*
mc_schem is a rust library to generate, load, manipulate and save minecraft schematic files.
Copyright (C) 2024  joseph

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::c_ffi::{
    CArrayView, CMapBox, CMapIterator, CMapKeyType, CMapKeyWrapper, CMapRef, CMapValueType,
    CMapValueWrapper, CStringView, KVRef,
};
use crate::region::{BlockEntity, PendingTick};
use fastnbt::Value;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap};
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::ptr::{null, null_mut};

impl CMapRef {
    pub fn key_value_type(&self) -> (CMapKeyType, CMapValueType) {
        return match self {
            CMapRef::StrStr(_) => (CMapKeyType::String, CMapValueType::String),
            CMapRef::StrValue(_) => (CMapKeyType::String, CMapValueType::NBT),
            CMapRef::PosBlockEntity(_) => (CMapKeyType::Pos, CMapValueType::BlockEntity),
            CMapRef::PosPendingTick(_) => (CMapKeyType::Pos, CMapValueType::PendingTickList),
        };
    }
}

impl CMapBox {
    pub fn to_c_map_ref(&self) -> CMapRef {
        return match self {
            CMapBox::StrStr(ss) => {
                type V = BTreeMap<String, String>;
                CMapRef::StrStr(ss.as_ref() as *const V as *mut V)
            }
            CMapBox::StrValue(sv) => {
                type V = HashMap<String, Value>;
                CMapRef::StrValue(sv.as_ref() as *const V as *mut V)
            }
            CMapBox::PosBlockEntity(pb) => {
                type V = HashMap<[i32; 3], BlockEntity>;
                CMapRef::PosBlockEntity(pb.as_ref() as *const V as *mut V)
            }
            CMapBox::PosPendingTick(pp) => {
                type V = HashMap<[i32; 3], Vec<PendingTick>>;
                CMapRef::PosPendingTick(pp.as_ref() as *const V as *mut V)
            }
            CMapBox::None => panic!("Trying to convert CMapBox::None to CMapRef"),
        };
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
extern "C" fn MC_SCHEM_create_map(
    key_t: CMapKeyType,
    val_t: CMapValueType,
    success: *mut bool,
) -> CMapBox {
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
        if key_t == CMapKeyType::Pos && val_t == CMapValueType::PendingTickList {
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
extern "C" fn MC_SCHEM_map_find(
    map: *const CMapRef,
    key_t: CMapKeyType,
    val_t: CMapValueType,
    key: CMapKeyWrapper,
    ok: *mut bool,
) -> CMapValueWrapper {
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
                debug_assert!(val_t == CMapValueType::PendingTickList);
                let map = &mut *(*map);
                let slice = match map.get_mut(&key.pos) {
                    Some(v) => v.as_slice(),
                    None => &[],
                };
                CMapValueWrapper {
                    pending_tick_view: ManuallyDrop::new(CArrayView::from_slice(slice)),
                }
            }
        };
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
        };
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
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_reserve(map: *mut CMapRef, new_capacity: usize) {
    unsafe {
        let map = &*map;
        match map {
            CMapRef::StrStr(_) => {}
            CMapRef::StrValue(map) => {
                let map = &mut **map;
                map.reserve(max(new_capacity, map.len()) - map.len());
            }
            CMapRef::PosBlockEntity(map) => {
                let map = &mut **map;
                map.reserve(max(new_capacity, map.len()) - map.len());
            }
            CMapRef::PosPendingTick(map) => {
                let map = &mut **map;
                map.reserve(max(new_capacity, map.len()) - map.len());
            }
        }
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn MC_SCHEM_map_iterator_first(
    map: *const CMapRef,
    key_t: CMapKeyType,
    val_t: CMapValueType,
    ok: *mut bool,
) -> CMapIterator {
    unsafe {
        let map = map as *mut CMapRef;
        let map = &mut *map;
        if map.key_value_type().0 != key_t || map.key_value_type().1 != val_t {
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
                CMapIterator::StrStr {
                    iter: map.iter_mut(),
                    deref,
                }
            }
            CMapRef::StrValue(map) => {
                debug_assert!(key_t == CMapKeyType::String);
                debug_assert!(val_t == CMapValueType::NBT);
                let map = &mut *(*map);
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::StrValue {
                    iter: map.iter_mut(),
                    deref,
                }
            }
            CMapRef::PosBlockEntity(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::BlockEntity);
                let map = &mut *(*map);
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::PosBlockEntity {
                    iter: map.iter_mut(),
                    deref,
                }
            }
            CMapRef::PosPendingTick(map) => {
                debug_assert!(key_t == CMapKeyType::Pos);
                debug_assert!(val_t == CMapValueType::PendingTickList);
                let map = &mut *(*map);
                let deref = KVRef::new(map.iter_mut().next());
                CMapIterator::PosPendingTick {
                    iter: map.iter_mut(),
                    deref,
                }
            }
        };
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn MC_SCHEM_map_iterator_end(
    map: *const CMapRef,
    key_t: CMapKeyType,
    val_t: CMapValueType,
    ok: *mut bool,
) -> CMapIterator {
    unsafe {
        let map = map as *mut CMapRef;
        let map = &mut *map;
        if (key_t, val_t) != map.key_value_type() {
            *ok = false;
            return CMapIterator::None;
        }
        *ok = true;
        return CMapIterator::None;
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
            CMapIterator::None => {}
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
            key: CMapKeyWrapper {
                string: CStringView::from(""),
            },
            value: CMapValueWrapper { string: null_mut() },
            has_value: false,
        };
        match it {
            CMapIterator::None => {
                return ret;
            }
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
                    ret.value.pending_tick_view =
                        ManuallyDrop::new(CArrayView::from_slice(&*(deref.value)));
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

// CMapIterator::None equals to CMapIterator::None,

impl CMapIterator {
    pub fn to_kv_pointer(&self) -> (*const c_void, *mut c_void) {
        return match self {
            CMapIterator::None => (null(), null_mut()),
            CMapIterator::StrStr { iter: _, deref } => {
                (deref.key as *const c_void, deref.value as *mut c_void)
            }
            CMapIterator::StrValue { iter: _, deref } => {
                (deref.key as *const c_void, deref.value as *mut c_void)
            }
            CMapIterator::PosBlockEntity { iter: _, deref } => {
                (deref.key as *const c_void, deref.value as *mut c_void)
            }
            CMapIterator::PosPendingTick { iter: _, deref } => {
                (deref.key as *const c_void, deref.value as *mut c_void)
            }
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_iterator_equal(a: *const CMapIterator, b: *const CMapIterator) -> bool {
    unsafe {
        let a = &*a;
        let b = &*b;

        return a.to_kv_pointer() == b.to_kv_pointer();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_iterator_length(it: *const CMapIterator) -> usize {
    unsafe {
        let it = &*it;
        return match it {
            CMapIterator::None => 0,
            CMapIterator::StrStr { iter, .. } => iter.len(),
            CMapIterator::StrValue { iter, .. } => iter.len(),
            CMapIterator::PosBlockEntity { iter, .. } => iter.len(),
            CMapIterator::PosPendingTick { iter, .. } => iter.len(),
        };
    }
}

type Callback = extern "C" fn(
    idx: usize,
    key: CMapKeyWrapper,
    value: CMapValueWrapper,
    custom_data: *mut c_void,
);

#[no_mangle]
#[allow(improper_ctypes_definitions)]
extern "C" fn MC_SCHEM_map_foreach(map: *const CMapRef, fun: Callback, custom_data: *mut c_void) {
    unsafe {
        let map = &*map;
        return match map {
            CMapRef::StrStr(map) => {
                let map = &mut *(*map);
                for (idx, (key, val)) in map.iter_mut().enumerate() {
                    fun(
                        idx,
                        CMapKeyWrapper {
                            string: CStringView::from(key),
                        },
                        CMapValueWrapper { string: val },
                        custom_data,
                    );
                }
            }
            CMapRef::StrValue(map) => {
                let map = &mut *(*map);
                for (idx, (key, val)) in map.iter_mut().enumerate() {
                    fun(
                        idx,
                        CMapKeyWrapper {
                            string: CStringView::from(key),
                        },
                        CMapValueWrapper { nbt: val },
                        custom_data,
                    );
                }
            }
            CMapRef::PosBlockEntity(map) => {
                let map = &mut *(*map);
                for (idx, (key, val)) in map.iter_mut().enumerate() {
                    fun(
                        idx,
                        CMapKeyWrapper { pos: *key },
                        CMapValueWrapper { block_entity: val },
                        custom_data,
                    );
                }
            }
            CMapRef::PosPendingTick(map) => {
                let map = &mut *(*map);
                for (idx, (key, val)) in map.iter_mut().enumerate() {
                    fun(
                        idx,
                        CMapKeyWrapper { pos: *key },
                        CMapValueWrapper {
                            pending_tick_view: ManuallyDrop::new(CArrayView::from_slice(
                                val.as_slice(),
                            )),
                        },
                        custom_data,
                    );
                }
            }
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_insert(map: *mut CMapRef, key: CMapKeyWrapper, value: CMapValueWrapper) {
    unsafe {
        let map = &*map;
        match map {
            CMapRef::StrStr(map) => {
                let map = &mut *(*map);
                map.insert(key.string.to_string(), (*value.string).clone());
            }
            CMapRef::StrValue(map) => {
                let map = &mut *(*map);
                map.insert(key.string.to_string(), (*value.nbt).clone());
            }
            CMapRef::PosBlockEntity(map) => {
                let map = &mut *(*map);
                map.insert(key.pos, (*value.block_entity).clone());
            }
            CMapRef::PosPendingTick(map) => {
                let map = &mut *(*map);
                map.insert(key.pos, value.pending_tick_view.to_vec());
            }
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_map_remove(
    map: *mut CMapRef,
    key: CMapKeyWrapper,
    has_value_before_erase_nullable: *mut bool,
) {
    unsafe {
        let map = &*map;
        let has_value_before_erase: bool;
        match map {
            CMapRef::StrStr(map) => {
                let map = &mut *(*map);
                has_value_before_erase = map.remove(key.string.to_str()).is_some();
            }
            CMapRef::StrValue(map) => {
                let map = &mut *(*map);
                has_value_before_erase = map.remove(key.string.to_str()).is_some();
            }
            CMapRef::PosBlockEntity(map) => {
                let map = &mut *(*map);
                has_value_before_erase = map.remove(&key.pos).is_some();
            }
            CMapRef::PosPendingTick(map) => {
                let map = &mut *(*map);
                has_value_before_erase = map.remove(&key.pos).is_some();
            }
        }

        if has_value_before_erase_nullable != null_mut() {
            *has_value_before_erase_nullable = has_value_before_erase;
        }
    }
}
