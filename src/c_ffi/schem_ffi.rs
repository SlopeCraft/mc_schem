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

use crate::block::Block;
use crate::c_ffi::{
    write_to_c_buffer, CLitematicaLoadOption, CLitematicaSaveOption, CMetadata, COption, CPosInt,
    CReader, CSchemLoadResult, CStringView, CVanillaStructureLoadOption,
    CVanillaStructureSaveOption, CWE12LoadOption, CWE13LoadOption, CWE13SaveOption, CWriter,
};
use crate::error::Error;
use crate::schem::{
    LitematicaLoadOption, LitematicaSaveOption, VanillaStructureLoadOption,
    VanillaStructureSaveOption, WorldEdit12LoadOption, WorldEdit13LoadOption,
    WorldEdit13SaveOption,
};
use crate::{Region, Schematic};
use std::ptr::{drop_in_place, null, slice_from_raw_parts};

#[no_mangle]
extern "C" fn MC_SCHEM_create_schem() -> Box<Schematic> {
    return Box::new(Schematic::new());
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_release_schem(b: *mut Box<Schematic>) {
    drop_in_place(b);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_schem(a: *mut Schematic, b: *mut Schematic) {
    std::mem::swap(&mut *a, &mut *b);
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
unsafe extern "C" fn MC_SCHEM_schem_load_litematica(
    mut src: CReader,
    option: *const CLitematicaLoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_litematica_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_litematica_file(
    filename: CStringView,
    option: *const CLitematicaLoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_litematica_file(filename.to_str(), &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_litematica_bytes(
    bytes: *const u8,
    length: usize,
    option: *const CLitematicaLoadOption,
) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_litematica_reader(bytes, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_vanilla_structure(
    mut src: CReader,
    option: *const CVanillaStructureLoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_vanilla_structure_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_vanilla_structure_file(
    filename: CStringView,
    option: *const CVanillaStructureLoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_vanilla_structure_file(
        filename.to_str(),
        &option,
    ));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_vanilla_structure_bytes(
    bytes: *const u8,
    length: usize,
    option: *const CVanillaStructureLoadOption,
) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_vanilla_structure_reader(bytes, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_13(
    mut src: CReader,
    option: *const CWE13LoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_13_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_13_file(
    filename: CStringView,
    option: *const CWE13LoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_13_file(
        filename.to_str(),
        &option,
    ));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_13_bytes(
    bytes: *const u8,
    length: usize,
    option: *const CWE13LoadOption,
) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::new(Schematic::from_world_edit_13_reader(bytes, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_12(
    mut src: CReader,
    option: *const CWE12LoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::from(Schematic::from_world_edit_12_reader(&mut src, &option));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_12_file(
    filename: CStringView,
    option: *const CWE12LoadOption,
) -> CSchemLoadResult {
    let option = (*option).to_option();
    return CSchemLoadResult::from(Schematic::from_world_edit_12_file(
        filename.to_str(),
        &option,
    ));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_load_world_edit_12_bytes(
    bytes: *const u8,
    length: usize,
    option: *const CWE12LoadOption,
) -> CSchemLoadResult {
    let bytes: &mut &[u8] = &mut &*slice_from_raw_parts(bytes, length);
    let option = (*option).to_option();
    return CSchemLoadResult::from(Schematic::from_world_edit_12_reader(bytes, &option));
}

#[no_mangle]
extern "C" fn MC_SCHEM_save_option_litematica_default() -> CLitematicaSaveOption {
    return CLitematicaSaveOption::from_option(&LitematicaSaveOption::default());
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_litematica(
    schem: *const Schematic,
    mut dst: CWriter,
    option: *const CLitematicaSaveOption,
) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_litematica_writer(&mut dst, &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_litematica_file(
    schem: *const Schematic,
    filename: CStringView,
    option: *const CLitematicaSaveOption,
) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_litematica_file(filename.to_str(), &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    };
}

#[no_mangle]
extern "C" fn MC_SCHEM_save_option_vanilla_structure_default() -> CVanillaStructureSaveOption {
    return CVanillaStructureSaveOption::from_option(&VanillaStructureSaveOption::default());
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_vanilla_structure(
    schem: *const Schematic,
    mut dst: CWriter,
    option: *const CVanillaStructureSaveOption,
) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_vanilla_structure_writer(&mut dst, &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_vanilla_structure_file(
    schem: *const Schematic,
    filename: CStringView,
    option: *const CVanillaStructureSaveOption,
) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_vanilla_structure_file(filename.to_str(), &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    };
}

#[no_mangle]
extern "C" fn MC_SCHEM_save_option_world_edit_13_default() -> CWE13SaveOption {
    return CWE13SaveOption::from_option(&WorldEdit13SaveOption::default());
}
#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_world_edit_13(
    schem: *const Schematic,
    mut dst: CWriter,
    option: *const CWE13SaveOption,
) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_world_edit_13_writer(&mut dst, &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_save_world_edit_13_file(
    schem: *const Schematic,
    filename: CStringView,
    option: *const CWE13SaveOption,
) -> Option<Box<Error>> {
    let option = (*option).to_option();
    return match (*schem).save_world_edit_13_file(filename.to_str(), &option) {
        Ok(_) => None,
        Err(e) => Some(Box::new(e)),
    };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_metadata(schem: *const Schematic) -> CMetadata {
    return CMetadata::new(&(*schem).metadata);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_set_metadata(schem: *mut Schematic, md: *const CMetadata) {
    (*schem).metadata = (*md).to_metadata();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_region_num(schem: *const Schematic) -> usize {
    return (*schem).regions.len();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_region(
    schem: *const Schematic,
    index: usize,
) -> *mut Region {
    return (*schem).regions.as_ptr().add(index) as *mut Region;
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_take_region(
    schem: *mut Schematic,
    index: usize,
) -> Box<Region> {
    return Box::new((*schem).regions.remove(index));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_insert_region_copy(
    schem: *mut Schematic,
    region: *const Region,
    index: usize,
) {
    (*schem).regions.insert(index, (*region).clone());
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_insert_region_move(
    schem: *mut Schematic,
    region_box: *mut Box<Region>,
    index: usize,
) {
    let mut region = Region::new();
    std::mem::swap(&mut region, &mut (*region_box));
    (*schem).regions.insert(index, region);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_block_indices_at(
    schem: *const Schematic,
    pos: CPosInt,
    num_blocks: *mut usize,
    dest: *mut u16,
    dest_capacity: usize,
) {
    let result = (*schem).block_indices_at(pos.pos);
    write_to_c_buffer(&result, num_blocks, dest, dest_capacity);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_blocks_at(
    schem: *const Schematic,
    pos: CPosInt,
    num_blocks: *mut usize,
    dest: *mut *const Block,
    dest_capacity: usize,
) {
    let result = (*schem).blocks_at(pos.pos);
    *num_blocks = result.len();

    if dest_capacity >= result.len() {
        for (idx, blk) in result.iter().enumerate() {
            *(dest.clone().add(idx)) = *blk as *const Block;
        }
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_first_block_index_at(
    schem: *const Schematic,
    pos: CPosInt,
) -> COption<u16> {
    let opt = (*schem).first_block_index_at(pos.pos);
    return COption::from(opt);
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_first_block_at(
    schem: *const Schematic,
    pos: CPosInt,
) -> *const Block {
    let opt = (*schem).first_block_at(pos.pos);
    return match opt {
        Some(blk) => blk as *const Block,
        None => null(),
    };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_shape(schem: *const Schematic) -> CPosInt {
    return CPosInt {
        pos: (*schem).shape(),
    };
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_volume(schem: *const Schematic) -> u64 {
    return (*schem).volume();
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_schem_get_total_blocks(
    schem: *const Schematic,
    include_air: bool,
) -> u64 {
    return (*schem).total_blocks(include_air);
}
