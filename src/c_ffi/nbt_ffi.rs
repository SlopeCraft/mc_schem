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
    CArrayView, CByteArrayView, CEnumNBTType, CIntArrayView, CLongArrayView, CMapRef, CNBTListView,
    CStringView, CValueBox,
};
use fastnbt::Value;
use std::collections::HashMap;
use std::ffi::{c_double, c_float};
use std::mem::swap;
use std::ptr::{drop_in_place, null, null_mut};

#[no_mangle]
extern "C" fn MC_SCHEM_create_nbt() -> CValueBox {
    return CValueBox::new(Value::Byte(0));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_release_nbt(nbt_box: *mut CValueBox) {
    drop_in_place(nbt_box);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_nbt(a: *mut Value, b: *mut Value) {
    swap(&mut *a, &mut *b);
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_type(nbt: *const Value) -> CEnumNBTType {
    unsafe {
        return match &*nbt {
            Value::Byte(_) => CEnumNBTType::Byte,
            Value::Short(_) => CEnumNBTType::Short,
            Value::Int(_) => CEnumNBTType::Int,
            Value::Long(_) => CEnumNBTType::Long,
            Value::Float(_) => CEnumNBTType::Float,
            Value::Double(_) => CEnumNBTType::Double,
            Value::ByteArray(_) => CEnumNBTType::ByteArray,
            Value::String(_) => CEnumNBTType::String,
            Value::List(_) => CEnumNBTType::List,
            Value::Compound(_) => CEnumNBTType::Compound,
            Value::IntArray(_) => CEnumNBTType::IntArray,
            Value::LongArray(_) => CEnumNBTType::LongArray,
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_byte(nbt: *const Value, ok: *mut bool) -> i8 {
    unsafe {
        return if let Value::Byte(val) = &*nbt {
            *ok = true;
            *val
        } else {
            *ok = false;
            0
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_short(nbt: *const Value, ok: *mut bool) -> i16 {
    unsafe {
        return if let Value::Short(val) = &*nbt {
            *ok = true;
            *val
        } else {
            *ok = false;
            0
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_int(nbt: *const Value, ok: *mut bool) -> i32 {
    unsafe {
        return if let Value::Int(val) = &*nbt {
            *ok = true;
            *val
        } else {
            *ok = false;
            0
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_long(nbt: *const Value, ok: *mut bool) -> i64 {
    unsafe {
        return if let Value::Long(val) = &*nbt {
            *ok = true;
            *val
        } else {
            *ok = false;
            0
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_float(nbt: *const Value, ok: *mut bool) -> c_float {
    unsafe {
        return if let Value::Float(val) = &*nbt {
            *ok = true;
            *val
        } else {
            *ok = false;
            0.0
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_double(nbt: *const Value, ok: *mut bool) -> c_double {
    unsafe {
        return if let Value::Double(val) = &*nbt {
            *ok = true;
            *val
        } else {
            *ok = false;
            0.0
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_string(nbt: *const Value, ok: *mut bool) -> *const String {
    unsafe {
        return if let Value::String(val) = &*nbt {
            *ok = true;
            val as *const String
        } else {
            *ok = false;
            null()
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_byte_array(nbt: *const Value, ok: *mut bool) -> CByteArrayView {
    unsafe {
        return if let Value::ByteArray(arr) = &*nbt {
            *ok = true;
            CArrayView::from_slice(arr.as_ref())
        } else {
            *ok = false;
            CArrayView::empty()
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_int_array(nbt: *const Value, ok: *mut bool) -> CIntArrayView {
    unsafe {
        return if let Value::IntArray(arr) = &*nbt {
            *ok = true;
            CArrayView::from_slice(arr.as_ref())
        } else {
            *ok = false;
            CArrayView::empty()
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_long_array(nbt: *const Value, ok: *mut bool) -> CLongArrayView {
    unsafe {
        return if let Value::LongArray(arr) = &*nbt {
            *ok = true;
            CArrayView::from_slice(arr.as_ref())
        } else {
            *ok = false;
            CArrayView::empty()
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_list(nbt: *const Value, ok: *mut bool) -> CNBTListView {
    unsafe {
        return if let Value::List(arr) = &*nbt {
            *ok = true;
            CArrayView::from_slice(arr.as_ref())
        } else {
            *ok = false;
            CArrayView::empty()
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_get_compound(nbt: *const Value, ok: *mut bool) -> CMapRef {
    unsafe {
        return if let Value::Compound(c) = &*nbt {
            *ok = true;
            CMapRef::StrValue(c as *const HashMap<String, Value> as *mut HashMap<String, Value>)
        } else {
            *ok = false;
            CMapRef::StrValue(null_mut())
        };
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_byte(nbt: *mut Value, val: i8) {
    unsafe {
        *nbt = Value::Byte(val);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_short(nbt: *mut Value, val: i16) {
    unsafe {
        *nbt = Value::Short(val);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_int(nbt: *mut Value, val: i32) {
    unsafe {
        *nbt = Value::Int(val);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_long(nbt: *mut Value, val: i64) {
    unsafe {
        *nbt = Value::Long(val);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_float(nbt: *mut Value, val: f32) {
    unsafe {
        *nbt = Value::Float(val);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_double(nbt: *mut Value, val: f64) {
    unsafe {
        *nbt = Value::Double(val);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_string(nbt: *mut Value, val: CStringView) {
    unsafe {
        let nbt = &mut *nbt;
        if let Value::String(s) = nbt {
            *s = val.to_string();
        } else {
            *nbt = Value::String(val.to_string());
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_byte_array(nbt: *mut Value, val: CByteArrayView) {
    unsafe {
        let nbt = &mut *nbt;
        if let Value::ByteArray(s) = nbt {
            *s = fastnbt::ByteArray::new(val.to_vec());
        } else {
            *nbt = Value::ByteArray(fastnbt::ByteArray::new(val.to_vec()));
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_int_array(nbt: *mut Value, val: CIntArrayView) {
    unsafe {
        let nbt = &mut *nbt;
        if let Value::IntArray(s) = nbt {
            *s = fastnbt::IntArray::new(val.to_vec());
        } else {
            *nbt = Value::IntArray(fastnbt::IntArray::new(val.to_vec()));
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_long_array(nbt: *mut Value, val: CLongArrayView) {
    unsafe {
        let nbt = &mut *nbt;
        if let Value::LongArray(s) = nbt {
            *s = fastnbt::LongArray::new(val.to_vec());
        } else {
            *nbt = Value::LongArray(fastnbt::LongArray::new(val.to_vec()));
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_list(nbt: *mut Value, val: CNBTListView) {
    unsafe {
        let nbt = &mut *nbt;
        if let Value::List(s) = nbt {
            *s = val.to_vec();
        } else {
            *nbt = Value::List(val.to_vec());
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_nbt_set_compound(nbt: *mut Value, val: CMapRef, ok: *mut bool) {
    unsafe {
        if let CMapRef::StrValue(val) = val {
            let val = &*val;
            *ok = true;
            if let Value::Compound(c) = &mut *nbt {
                *c = val.clone();
            } else {
                *nbt = Value::Compound(val.clone());
            }
        } else {
            *ok = false;
        }
    }
}
