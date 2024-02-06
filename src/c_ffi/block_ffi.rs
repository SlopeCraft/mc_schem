use std::collections::BTreeMap;
use std::ptr::drop_in_place;
use crate::block::Block;
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
