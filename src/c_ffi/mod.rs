use std::ffi::c_char;
use std::mem::size_of;
use std::ptr::drop_in_place;
use crate::schem::Schematic;

#[no_mangle]
pub extern "C" fn MC_SCHEM_version_string() -> *const c_char {
    return "0.1.0\0".as_ptr() as *const c_char;
}

#[no_mangle]
pub extern "C" fn MC_SCHEM_version_major() -> u16 {
    return 0;
}

#[no_mangle]
pub extern "C" fn MC_SCHEM_version_minor() -> u16 {
    return 1;
}

#[no_mangle]
pub extern "C" fn MC_SCHEM_version_patch() -> u16 {
    return 0;
}

#[no_mangle]
pub extern "C" fn MC_SCHEM_version_tweak() -> u16 {
    return 0;
}

#[repr(C, align(8))]
struct SchemString {
    begin: *const c_char,
    end: *const c_char,
}

impl SchemString {
    fn new(src: &str) -> SchemString {
        unsafe {
            let begin = src.as_ptr() as *const c_char;
            return SchemString {
                begin,
                end: begin.add(src.len()),
            };
        }
    }
}

#[repr(C, align(8))]
struct CSchematic {
    value: Box<Schematic>,
}

#[no_mangle]
pub extern "C" fn MC_SCHEM_create_schem() -> CSchematic {
    return CSchematic {
        value: Box::new(Schematic::new()),
    };
}

#[no_mangle]
pub unsafe extern "C" fn MC_SCHEM_destroy_schem(ptr: *mut CSchematic) {
    let cs = &mut *ptr;
    drop_in_place(&mut cs.value);
}