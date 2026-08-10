use std::ffi::c_double;
use crate::region::Entity;

// void mc_schem_entity_get_position(const entity *, int32_t* x, int32_t* y,int32_t* z, double* fp_x, double* fp_y,double* fp_z)
#[no_mangle]
pub unsafe extern "C" fn mc_schem_entity_get_position(
    entity: *const Entity,
    dest_x: *mut i32,
    dest_y: *mut i32,
    dest_z: *mut i32,
    dest_fp_x: *mut c_double,
    dest_fp_y: *mut c_double,
    dest_fp_z: *mut c_double,
) {
    {
        let [x, y, z] = (*entity).block_pos;
        *dest_x = x;
        *dest_y = y;
        *dest_z = z;
    }
    {
        let [x, y, z] = (*entity).position;
        *dest_fp_x = x as c_double;
        *dest_fp_y = y as c_double;
        *dest_fp_z = z as c_double;
    }
}
