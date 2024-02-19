use std::ptr::{drop_in_place, slice_from_raw_parts};
use crate::c_ffi::{CLitematicaLoadOption, CLitematicaSaveOption, CMetadata, CReader, CSchemLoadResult, CStringView, CVanillaStructureLoadOption, CVanillaStructureSaveOption, CWE12LoadOption, CWE13LoadOption, CWE13SaveOption, CWriter};
use crate::error::Error;
use crate::schem::{LitematicaLoadOption, LitematicaSaveOption, VanillaStructureLoadOption, VanillaStructureSaveOption, WorldEdit12LoadOption, WorldEdit13LoadOption, WorldEdit13SaveOption};
use crate::Schematic;

#[no_mangle]
extern "C" fn MC_SCHEM_create_schem() -> Box<Schematic> {
    return Box::new(Schematic::new());
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_release_schem(b: *mut Box<Schematic>) {
    drop_in_place(b);
}


#[no_mangle]
extern "C" fn MC_SCHEM_load_option_litematica_default() -> CLitematicaLoadOption {
    return CLitematicaLoadOption::from_option(&LitematicaLoadOption::default());
}

#[no_mangle]
extern "C" fn MC_SCHEM_load_option_vanilla_structure_default() -> CVanillaStructureLoadOption {
    return CVanillaStructureLoadOption::from_option(&VanillaStructureLoadOption::default());
}

#[no_mangle]
extern "C" fn MC_SCHEM_load_option_world_edit_13_default() -> CWE13LoadOption {
    return CWE13LoadOption::from_option(&WorldEdit13LoadOption::default());
}

#[no_mangle]
extern "C" fn MC_SCHEM_load_option_world_edit_12_default() -> CWE12LoadOption {
    return CWE12LoadOption::from_option(&WorldEdit12LoadOption::default());
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_litematica(mut src: CReader,
                                                    option: *const CLitematicaLoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_litematica_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_litematica_file(filename: CStringView,
                                                         option: *const CLitematicaLoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_litematica_file(filename.to_str(), &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_litematica_bytes(
    bytes: *const u8, length: usize, option: *const CLitematicaLoadOption) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_litematica_reader(bytes, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_vanilla_structure(mut src: CReader,
                                                           option: *const CVanillaStructureLoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_vanilla_structure_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_vanilla_structure_file(filename: CStringView,
                                                                option: *const CVanillaStructureLoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_vanilla_structure_file(filename.to_str(), &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_vanilla_structure_bytes(
    bytes: *const u8, length: usize, option: *const CVanillaStructureLoadOption) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_vanilla_structure_reader(bytes, &option));
}


#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_13(mut src: CReader,
                                                       option: *const CWE13LoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_13_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_13_file(filename: CStringView,
                                                            option: *const CWE13LoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_13_file(filename.to_str(), &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_13_bytes(
    bytes: *const u8, length: usize, option: *const CWE13LoadOption) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_13_reader(bytes, &option));
}


#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_12(mut src: CReader,
                                                       option: *const CWE12LoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_12_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_12_file(filename: CStringView,
                                                            option: *const CWE12LoadOption) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_12_file(filename.to_str(), &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_12_bytes(
    bytes: *const u8, length: usize, option: *const CWE12LoadOption) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_12_reader(bytes, &option));
}

#[no_mangle]
extern "C" fn MC_SCHEM_save_option_litematica_default() -> CLitematicaSaveOption {
    return CLitematicaSaveOption::from_option(&LitematicaSaveOption::default());
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_litematica(schem: *const Schematic, mut dst: CWriter, option: *const CLitematicaSaveOption) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_litematica_writer(&mut dst, &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_litematica_file(schem: *const Schematic, filename: CStringView, option: *const CLitematicaSaveOption) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_litematica_file(filename.to_str(), &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_save_option_vanilla_structure_default() -> CVanillaStructureSaveOption {
    return CVanillaStructureSaveOption::from_option(&VanillaStructureSaveOption::default());
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_vanilla_structure(schem: *const Schematic, mut dst: CWriter, option: *const CVanillaStructureSaveOption) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_vanilla_structure_writer(&mut dst, &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_vanilla_structure_file(schem: *const Schematic, filename: CStringView, option: *const CVanillaStructureSaveOption) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_vanilla_structure_file(filename.to_str(), &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_save_option_world_edit_13_default() -> CWE13SaveOption {
    return CWE13SaveOption::from_option(&WorldEdit13SaveOption::default());
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_world_edit_13(schem: *const Schematic, mut dst: CWriter, option: *const CWE13SaveOption) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_world_edit_13_writer(&mut dst, &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_world_edit_13_file(schem: *const Schematic, filename: CStringView, option: *const CWE13SaveOption) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_world_edit_13_file(filename.to_str(), &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_metadata(schem: *const Schematic) -> CMetadata {
    return CMetadata::new(&(*schem).metadata);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_set_metadata(schem: *mut Schematic, md: *const CMetadata) {
    (*schem).metadata = (*md).to_metadata();
}