use crate::c_ffi::rust_string_receiver;
use crate::schem::MetaDataIR;
use crate::Error;
use std::ptr::null_mut;

// meta_data_ir* mc_schem_create_meta_data_ir(int32_t data_version, error** dest_err_non_null)
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_meta_data_ir(
    data_version: i32,
    err_dest: *mut *mut Error,
) -> *mut MetaDataIR {
    let result = MetaDataIR::from_data_version_i32(data_version);
    *err_dest = null_mut();

    match result {
        Ok(ir) => {
            let b = Box::from(ir);
            Box::into_raw(b)
        }
        Err(err) => {
            let b = Box::from(err);
            *err_dest = Box::into_raw(b);
            null_mut()
        }
    }
}

// int32_t mc_schem_meta_data_ir_get_mc_data_version(const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_mc_data_version(ir: *const MetaDataIR) -> i32 {
    (*ir).mc_data_version
}
// int64_t mc_schem_meta_data_ir_get_time_created(const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_time_created(ir: *const MetaDataIR) -> i64 {
    (*ir).time_created
}
// int64_t mc_schem_meta_data_ir_get_time_modified(const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_time_modified(ir: *const MetaDataIR) -> i64 {
    (*ir).time_modified
}
// void mc_schem_meta_data_ir_get_author(const meta_data_ir*, const rust_string_receiver* dest);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_author(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((*ir).author.as_str());
}
// void mc_schem_meta_data_ir_get_name(const meta_data_ir*, const rust_string_receiver* dest);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_name(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((*ir).name.as_str())
}
// int32_t mc_schem_meta_data_ir_get_litematica_version(const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_litematica_version(
    ir: *const MetaDataIR,
) -> i32 {
    (*ir).litematica_version
}
// int32_t mc_schem_meta_data_ir_get_litematica_subversion(const meta_data_ir*, bool* dest_exist_non_null);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_litematica_subversion(
    ir: *const MetaDataIR,
    dest_exist: *mut bool,
) -> i32 {
    *dest_exist = (*ir).litematica_subversion.is_some();
    (*ir).litematica_subversion.unwrap_or_else(|| i32::MAX)
}
// int32_t mc_schem_meta_data_ir_get_schem_version(const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_version(ir: *const MetaDataIR) -> i32 {
    (*ir).schem_version
}
// void mc_schem_meta_data_ir_get_schem_offset(const meta_data_ir*, int32_t* dest_x, int32_t* dest_y, int32_t* dest_z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_offset(
    ir: *const MetaDataIR,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
) {
    let [x, y, z] = (*ir).schem_offset;
    *dest_x = x;
    *dest_y = y;
    *dest_z = z;
}
// bool mc_schem_meta_data_ir_get_schem_we_offset(const meta_data_ir*, int32_t* dest_x, int32_t* dest_y, int32_t* dest_z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_we_offset(
    ir: *const MetaDataIR,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
) -> bool {
    if let Some([x, y, z]) = (*ir).schem_we_offset {
        *dest_x = x;
        *dest_y = y;
        *dest_z = z;
        return true;
    }
    *dest_x = 0;
    *dest_y = 0;
    *dest_z = 0;
    (*ir).schem_we_offset.is_some()
}
// bool mc_schem_meta_data_ir_get_schem_world_edit_version(const meta_data_ir*, const rust_string_receiver*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_world_edit_version(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) -> bool {
    if let Some(s) = &(*ir).schem_world_edit_version {
        (*dest).receive(s.as_str());
        return true;
    }
    (*dest).receive("");
    false
}
// bool mc_schem_meta_data_ir_get_schem_editing_platform(const meta_data_ir*, const rust_string_receiver*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_editing_platform(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) -> bool {
    if let Some(s) = &(*ir).schem_editing_platform {
        (*dest).receive(s.as_str());
        return true;
    }
    (*dest).receive("");
    false
}
// bool mc_schem_meta_data_ir_get_schem_origin(const meta_data_ir*, int32_t* dest_x, int32_t* dest_y, int32_t* dest_z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_origin(
    ir: *const MetaDataIR,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
) -> bool {
    if let Some([x, y, z]) = (*ir).schem_origin {
        *dest_x = x;
        *dest_y = y;
        *dest_z = z;
        return true;
    }
    *dest_x = 0;
    *dest_y = 0;
    *dest_z = 0;
    false
}
// void mc_schem_meta_data_ir_get_schem_material(const meta_data_ir*, const rust_string_receiver*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_meta_data_ir_get_schem_material(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) {
    (*dest).receive(&(*ir).schem_material);
}
