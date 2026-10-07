// [[nodiscard]] schematic* mc_schem_create_schematic();

use std::ptr::{null, null_mut};
use crate::Region;
use crate::schem::{MetaDataIR, Schematic};

#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_schematic() -> *mut Schematic {
    Box::into_raw(Box::new(Schematic::new()))
}


// [[nodiscard]] const meta_data_ir* mc_schem_schematic_get_meta_data(const schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_meta_data(schem: *const Schematic) -> *const MetaDataIR {
    &(schem.as_ref_unchecked().metadata)
}
// [[nodiscard]] meta_data_ir* mc_schem_schematic_get_meta_data_mut(schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_meta_data_mut(schem: *mut Schematic) -> *mut MetaDataIR {
    &mut (schem.as_mut_unchecked().metadata)
}
// /// Deep copy given metadata, move old value to heap and return
// meta_data_ir* mc_schem_schematic_set_meta_data(schematic*, const meta_data_ir*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_set_meta_data(schem: *mut Schematic, new_ir: *const MetaDataIR) -> *mut MetaDataIR {
    let mut val = new_ir.as_ref_unchecked().clone();
    std::mem::swap(&mut val, &mut (schem.as_mut_unchecked().metadata));
    Box::into_raw(Box::from(val))
}
// [[nodiscard]] size_t mc_schem_schematic_get_regions_count(const schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_regions_count(schem: *const Schematic) -> usize {
    schem.as_ref_unchecked().regions.len()
}

// [[nodiscard]] const region* mc_schem_schematic_get_region(const schematic*,size_t idx);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_region(schem: *const Schematic, idx: usize) -> *const Region {
    if let Some(r) = schem.as_ref_unchecked().regions.get(idx) {
        return r;
    }
    null()
}
// [[nodiscard]] region* mc_schem_schematic_get_region_mut(schematic*,size_t idx);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_get_region_mut(schem: *mut Schematic, idx: usize) -> *mut Region {
    if let Some(r) = schem.as_mut_unchecked().regions.get_mut(idx) {
        return r;
    }
    null_mut()
}
// /// Remove a region from schematic, move to heap and return its pointer.
// /// Transfer ownership.
// [[nodiscard]] region* mc_schem_schematic_remove_region(schematic*,size_t idx);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_remove_region(schem: *mut Schematic, idx: usize) -> *mut Schematic {
    if let Some(r) = schem.as_mut_unchecked().regions.remove(idx) {
        return Box::into_raw(Box::from(r));
    }
    null_mut()
}
// /// Remove all regions from schematic
// void mc_schem_schematic_clear_all_regions(schematic*);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_clear_all_regions(schem: *mut Schematic) {
    schem.as_mut_unchecked().regions.clear()
}
// /// Deep copy new_region into given index, return its pointer in schematic. If
// /// index out of range (for example, insert to index=4 with only 3 regions,
// /// nothing will be done and returns nullptr)
// region* mc_schem_schematic_insert_region(schematic*, const region* new_region, size_t index);

#[no_mangle]
pub unsafe extern "C" fn mc_schem_schematic_insert_region(schem: *mut Schematic, region: *const Region, idx: usize) -> *mut Region {
    if idx > schem.as_mut_unchecked().regions.len() {
        return null_mut();
    }
    schem.as_mut_unchecked().regions.insert_mut(idx, region.as_ref_unchecked().clone())
}