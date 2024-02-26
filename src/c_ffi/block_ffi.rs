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

use std::collections::BTreeMap;
use std::ffi::c_char;
use std::intrinsics::copy_nonoverlapping;
use std::mem::swap;
use std::ptr::{drop_in_place, null_mut};
use crate::block::{Block, BlockIdParseError};
use crate::c_ffi::{CMapRef, CStringView};

#[no_mangle]
extern "C" fn MC_SCHEM_create_block() -> Box<Block> {
    return Box::new(Block::empty_block());
}

#[no_mangle]
extern "C" fn MC_SCHEM_release_block(block_box: *mut Box<Block>) {
    unsafe {
        drop_in_place(block_box);
    }
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_swap_block(a: *mut Block, b: *mut Block) {
    swap(&mut *a, &mut *b);
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_get_namespace(block: *const Block) -> CStringView {
    unsafe {
        return CStringView::from((*block).namespace.as_str());
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_get_id(block: *const Block) -> CStringView {
    unsafe {
        return CStringView::from((*block).id.as_str());
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_get_attributes(block: *const Block) -> CMapRef {
    unsafe {
        let block = &*block;
        return CMapRef::StrStr(&block.attributes as *const BTreeMap<String, String> as *mut BTreeMap<String, String>);
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_set_namespace(block: *mut Block, namespace: CStringView) {
    unsafe {
        (*block).namespace = namespace.to_string();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_set_id(block: *mut Block, id: CStringView) {
    unsafe {
        (*block).id = id.to_string();
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_set_attributes(block: *mut Block, map: CMapRef, ok: *mut bool) {
    unsafe {
        if let CMapRef::StrStr(map) = map {
            let map = &*map;
            *ok = true;
            let block = &mut *block;
            block.attributes = map.clone();
        } else {
            *ok = false;
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_parse_block(id: CStringView, block: *mut Block, error_nullable: *mut BlockIdParseError) -> bool {
    unsafe {
        let block = &mut *block;
        return match Block::from_id(id.to_str()) {
            Ok(blk) => {
                *block = blk;
                true
            },
            Err(e) => {
                if error_nullable != null_mut() {
                    *error_nullable = e;
                }
                false
            }
        }
    }
}

#[no_mangle]
extern "C" fn MC_SCHEM_block_to_full_id(block: *const Block,
                                        dest: *mut c_char,
                                        dest_capacity: usize,
                                        id_length: *mut usize) {
    unsafe {
        let block = &*block;
        let mut id = block.full_id();
        id.push('\0');
        let required_bytes = id.as_bytes().len();
        *id_length = required_bytes;

        if dest == null_mut() || dest_capacity < required_bytes {
            return;
        }

        copy_nonoverlapping(id.as_ptr() as *const c_char, dest, id.as_bytes().len());
    }
}