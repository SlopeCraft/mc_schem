use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use fastnbt::Value;
use crate::c_ffi::{CEnumNBTType, CMapRef, CNBTValue, RsObjWrapper, SchemString};

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_create_scalar(
    tag_type: CEnumNBTType, value: *const c_void, dst_success: *mut bool) -> CNBTValue {
    let val: Value;
    let mut success: bool = true;
    unsafe {
        match tag_type {
            CEnumNBTType::MC_SCHEM_nbt_type_byte => val = Value::from(*(value as *const i8)),
            CEnumNBTType::MC_SCHEM_nbt_type_short => val = Value::from(*(value as *const i16)),
            CEnumNBTType::MC_SCHEM_nbt_type_int => val = Value::from(*(value as *const i32)),
            CEnumNBTType::MC_SCHEM_nbt_type_long => val = Value::from(*(value as *const i64)),
            CEnumNBTType::MC_SCHEM_nbt_type_float => val = Value::from(*(value as *const f32)),
            CEnumNBTType::MC_SCHEM_nbt_type_double => val = Value::from(*(value as *const f64)),
            _ => {
                val = Value::from(0i8);
                success = false;
            }
        }
    }
    unsafe {
        if dst_success != null_mut() {
            *dst_success = success;
        }
    }
    if !success {
        return CNBTValue::Ref(null_mut());
    }

    return CNBTValue::Owned(Box::new(val));
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_release_value(value: *mut CNBTValue) {
    if value == null_mut() {
        return;
    }
    unsafe {
        let val_ref = &mut *value;
        val_ref.release();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_type(value: *const CNBTValue) -> CEnumNBTType {
    unsafe {
        let val_ref = &*value;
        let val_ref = val_ref.get_ref();

        return match val_ref {
            Value::Byte(_) => CEnumNBTType::MC_SCHEM_nbt_type_byte,
            Value::Short(_) => CEnumNBTType::MC_SCHEM_nbt_type_short,
            Value::Int(_) => CEnumNBTType::MC_SCHEM_nbt_type_int,
            Value::Long(_) => CEnumNBTType::MC_SCHEM_nbt_type_long,
            Value::Float(_) => CEnumNBTType::MC_SCHEM_nbt_type_float,
            Value::Double(_) => CEnumNBTType::MC_SCHEM_nbt_type_double,
            Value::ByteArray(_) => CEnumNBTType::MC_SCHEM_nbt_type_byte_array,
            Value::String(_) => CEnumNBTType::MC_SCHEM_nbt_type_string,
            Value::List(_) => CEnumNBTType::MC_SCHEM_nbt_type_list,
            Value::Compound(_) => CEnumNBTType::MC_SCHEM_nbt_type_compound,
            Value::IntArray(_) => CEnumNBTType::MC_SCHEM_nbt_type_int_array,
            Value::LongArray(_) => CEnumNBTType::MC_SCHEM_nbt_type_long_array,
        };
    }
}


#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_scalar(value: *const CNBTValue, dest: *mut c_void, dest_capacity: usize) -> bool {
    if dest == null_mut() {
        return false;
    }

    unsafe {
        let val_ref = &*value;
        let val_ref = val_ref.get_ref();

        return match val_ref {
            Value::Byte(val) => {
                if dest_capacity < size_of::<bool>() {
                    return false;
                }
                *(dest as *mut i8) = *val;
                true
            }
            Value::Short(val) => {
                if dest_capacity < size_of::<bool>() {
                    return false;
                }
                *(dest as *mut i16) = *val;
                true
            }
            Value::Int(val) => {
                if dest_capacity < size_of::<bool>() {
                    return false;
                }
                *(dest as *mut i32) = *val;
                true
            }
            Value::Long(val) => {
                if dest_capacity < size_of::<bool>() {
                    return false;
                }
                *(dest as *mut i64) = *val;
                true
            }
            Value::Float(val) => {
                if dest_capacity < size_of::<bool>() {
                    return false;
                }
                *(dest as *mut f32) = *val;
                true
            }
            Value::Double(val) => {
                if dest_capacity < size_of::<bool>() {
                    return false;
                }
                *(dest as *mut f64) = *val;
                true
            }
            _ => {
                false
            }
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_length(value: *const CNBTValue, dst_length: *mut usize) -> bool {
    unsafe {
        let val_ref = &*value;
        let val_ref = val_ref.get_ref();

        let length: usize;
        match val_ref {
            Value::ByteArray(ba) => length = ba.len(),
            Value::String(s) => length = s.len(),
            Value::List(l) => length = l.len(),
            Value::Compound(c) => length = c.len(),
            Value::IntArray(ia) => length = ia.len(),
            Value::LongArray(la) => length = la.len(),
            _ => {
                //length = 0;
                return false;
            }
        }

        if dst_length == null_mut() {
            return false;
        }
        *dst_length = length;
    }

    return true;
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_list_element_const(
    list: *const CNBTValue, index: usize) -> CNBTValue {
    if list == null_mut() {
        return CNBTValue::Ref(null_mut());
    }

    let list_ref;
    unsafe {
        let val_ref = &*list;
        let val_ref = val_ref.get_ref();
        match &val_ref {
            Value::List(l_ref) => list_ref = l_ref,
            _ => return CNBTValue::Ref(null_mut()),
        }
    }
    if index >= list_ref.len() { return CNBTValue::Ref(null_mut()); }

    let element_ptr: *mut Value = list_ref.as_ptr().wrapping_add(index) as *mut Value;
    return CNBTValue::Ref(element_ptr);
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_list_element_mut(
    list: *mut CNBTValue, index: usize) -> CNBTValue {
    return MC_SCHEM_nbt_get_list_element_const(list, index);
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_string(
    list: *const CNBTValue, dest: *mut SchemString) -> bool {
    unsafe {
        let val_ref = &*list;
        let val_ref = val_ref.get_ref();
        return match val_ref {
            Value::String(s) => {
                *dest = SchemString::new(&s);
                true
            }
            _ => false,
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_scalar_array_const(
    tag: *const CNBTValue,
    dest_ptr: *mut *const c_void,
    dest_num_elements: *mut usize) -> bool {
    unsafe {
        let val_ref = &*tag;
        let val_ref = val_ref.get_ref();

        let data: *const c_void;
        let len: usize;
        match val_ref {
            Value::ByteArray(a) => {
                data = a.as_ptr() as *const c_void;
                len = a.len();
            }
            Value::IntArray(a) => {
                data = a.as_ptr() as *const c_void;
                len = a.len();
            }
            Value::LongArray(a) => {
                data = a.as_ptr() as *const c_void;
                len = a.len();
            }
            _ => {
                return false;
            }
        }

        if dest_ptr != null_mut() {
            *dest_ptr = data;
        }
        if dest_num_elements != null_mut() {
            *dest_num_elements = len;
        }
    }
    return true;
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_scalar_array_mut(
    tag: *mut CNBTValue,
    dest_ptr: *mut *mut c_void,
    dest_num_elements: *mut usize) -> bool {
    return MC_SCHEM_nbt_get_scalar_array_const(tag, dest_ptr as *mut *const c_void, dest_num_elements);
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_compound_const(tag: *const CNBTValue, ok: *mut bool) -> CMapRef {
    unsafe {
        let tag = (*tag).get_ref();
        if let Value::Compound(map) = tag {
            if ok != null_mut() { *ok = true; }
            return CMapRef::StrValue(map as *const HashMap<String, Value> as *mut HashMap<String, Value>);
        }
        if ok != null_mut() { *ok = false; }
        return CMapRef::StrValue(null_mut());
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_compound_mut(tag: *mut CNBTValue, ok: *mut bool) -> CMapRef {
    return MC_SCHEM_nbt_get_compound_const(tag, ok);
}