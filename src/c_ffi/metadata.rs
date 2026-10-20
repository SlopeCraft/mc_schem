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
use crate::c_ffi::rust_string_receiver;
use crate::schem::MetaDataIR;
use crate::Error;
use std::ffi::{c_char, CStr};
use std::ptr::{null, null_mut};

// metadata_ir* mc_schem_create_metadata_ir(int32_t data_version, error** dest_err_non_null)
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_metadata_ir(
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

// int32_t mc_schem_metadata_ir_get_mc_data_version(const metadata_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_mc_data_version(ir: *const MetaDataIR) -> i32 {
    (*ir).mc_data_version
}
// int64_t mc_schem_metadata_ir_get_time_created(const metadata_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_time_created(ir: *const MetaDataIR) -> i64 {
    (*ir).time_created
}
// int64_t mc_schem_metadata_ir_get_time_modified(const metadata_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_time_modified(ir: *const MetaDataIR) -> i64 {
    (*ir).time_modified
}
// void mc_schem_metadata_ir_get_author(const metadata_ir*, const rust_string_receiver* dest);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_author(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((&*ir).author.as_str());
}
// void mc_schem_metadata_ir_get_name(const metadata_ir*, const rust_string_receiver* dest);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_name(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) {
    (*dest).receive((&*ir).name.as_str())
}
// int32_t mc_schem_metadata_ir_get_litematica_version(const metadata_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_litematica_version(ir: *const MetaDataIR) -> i32 {
    (*ir).litematica_version
}
// int32_t mc_schem_metadata_ir_get_litematica_subversion(const metadata_ir*, bool* dest_exist_non_null);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_litematica_subversion(
    ir: *const MetaDataIR,
    dest_exist: *mut bool,
) -> i32 {
    *dest_exist = (*ir).litematica_subversion.is_some();
    (*ir).litematica_subversion.unwrap_or_else(|| i32::MAX)
}
// int32_t mc_schem_metadata_ir_get_schem_version(const metadata_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_version(ir: *const MetaDataIR) -> i32 {
    (*ir).schem_version
}
// void mc_schem_metadata_ir_get_schem_offset(const metadata_ir*, int32_t* dest_x, int32_t* dest_y, int32_t* dest_z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_offset(
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
// bool mc_schem_metadata_ir_get_schem_we_offset(const metadata_ir*, int32_t* dest_x, int32_t* dest_y, int32_t* dest_z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_we_offset(
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
// bool mc_schem_metadata_ir_get_schem_world_edit_version(const metadata_ir*, const rust_string_receiver*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_world_edit_version(
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
// bool mc_schem_metadata_ir_get_schem_editing_platform(const metadata_ir*, const rust_string_receiver*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_editing_platform(
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
// bool mc_schem_metadata_ir_get_schem_origin(const metadata_ir*, int32_t* dest_x, int32_t* dest_y, int32_t* dest_z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_origin(
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
// void mc_schem_metadata_ir_get_schem_material(const metadata_ir*, const rust_string_receiver*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_get_schem_material(
    ir: *const MetaDataIR,
    dest: *const rust_string_receiver,
) {
    (*dest).receive(&(*ir).schem_material);
}

// [[nodiscard]] error* mc_schem_metadata_ir_set_mc_data_version(metadata_ir*,  int32_t version);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_mc_data_version(
    ir: *mut MetaDataIR,
    data_version: i32,
) -> *mut Error {
    ir.as_mut_unchecked().mc_data_version = data_version;
    // Maybe later will check for valid version
    null_mut()
}
// void mc_schem_metadata_ir_set_time_created(metadata_ir*, int64_t time);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_time_created(ir: *mut MetaDataIR, time: i64) {
    ir.as_mut_unchecked().time_created = time;
}
// void mc_schem_metadata_ir_set_time_modified(metadata_ir*, int64_t time);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_time_modified(ir: *mut MetaDataIR, time: i64) {
    ir.as_mut_unchecked().time_modified = time;
}
// void mc_schem_metadata_ir_set_author(metadata_ir*, const char*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_author(
    ir: *mut MetaDataIR,
    author: *const c_char,
) {
    let author = CStr::from_ptr(author);
    ir.as_mut_unchecked().author = author.to_string_lossy().to_string();
}
// void mc_schem_metadata_ir_set_name(metadata_ir*, const char*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_name(ir: *mut MetaDataIR, name: *const c_char) {
    ir.as_mut_unchecked().name = CStr::from_ptr(name).to_string_lossy().to_string();
}
// void mc_schem_metadata_ir_set_litematica_version(metadata_ir*, int32_t);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_litematica_version(
    ir: *mut MetaDataIR,
    val: i32,
) {
    ir.as_mut_unchecked().litematica_version = val;
}
// if exist=false, ignore subver; otherwise use subver
// void mc_schem_metadata_ir_set_litematica_subversion(metadata_ir*, int32_t subver, bool exist);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_litematica_subversion(
    ir: *mut MetaDataIR,
    val: i32,
    exist: bool,
) {
    ir.as_mut_unchecked().litematica_subversion = if exist { Some(val) } else { None };
}
// void mc_schem_metadata_ir_set_schem_version(metadata_ir*, int32_t);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_schem_version(ir: *mut MetaDataIR, val: i32) {
    ir.as_mut_unchecked().schem_version = val;
}
// void mc_schem_metadata_ir_set_schem_offset(metadata_ir*, int32_t x, int32_t y, int32_t z);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_schem_offset(
    ir: *mut MetaDataIR,
    x: i32,
    y: i32,
    z: i32,
) {
    ir.as_mut_unchecked().schem_offset = [x, y, z];
}
// void mc_schem_metadata_ir_set_we_offset(metadata_ir*, int32_t x, int32_t y, int32_t z, bool exist);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_we_offset(
    ir: *mut MetaDataIR,
    x: i32,
    y: i32,
    z: i32,
    exist: bool,
) {
    ir.as_mut_unchecked().schem_we_offset = if exist { Some([x, y, z]) } else { None };
}
// // if we_version=null, erase this tag; otherwise set to we_
// void mc_schem_metadata_ir_set_schem_world_edit_version( metadata_ir*, const char* we_version_nullable);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_schem_world_edit_version(
    ir: *mut MetaDataIR,
    we_version_nullable: *const c_char,
) {
    ir.as_mut_unchecked().schem_world_edit_version = if we_version_nullable == null() {
        None
    } else {
        Some(
            CStr::from_ptr(we_version_nullable)
                .to_string_lossy()
                .to_string(),
        )
    };
}
// void mc_schem_metadata_ir_set_schem_editing_platform( metadata_ir*, const char* we_editing_platform_nullable);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_schem_editing_platform(
    ir: *mut MetaDataIR,
    editing_plat: *const c_char,
) {
    ir.as_mut_unchecked().schem_editing_platform = if editing_plat == null() {
        None
    } else {
        Some(CStr::from_ptr(editing_plat).to_string_lossy().to_string())
    };
}
// void mc_schem_metadata_ir_set_schem_origin(metadata_ir*, int32_t x, int32_t y, int32_t z, bool exist);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_schem_origin(
    ir: *mut MetaDataIR,
    x: i32,
    y: i32,
    z: i32,
    exist: bool,
) {
    ir.as_mut_unchecked().schem_origin = if exist { Some([x, y, z]) } else { None };
}
// void mc_schem_metadata_ir_set_schem_material(metadata_ir*, const char*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_metadata_ir_set_schem_material(
    ir: *mut MetaDataIR,
    schem_mat: *const c_char,
) {
    ir.as_mut_unchecked().schem_material = CStr::from_ptr(schem_mat).to_string_lossy().to_string();
}
