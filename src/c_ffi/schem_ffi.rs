use std::mem::size_of;
use crate::schem::Schematic;
use static_assertions as sa;
use crate::c_ffi::CSchematic;



sa::const_assert!(size_of::<CSchematic>()==2*size_of::<usize>());

#[no_mangle]
extern "C" fn MC_SCHEM_create_schem() -> CSchematic {
    return CSchematic::Owned(Box::new(Schematic::new()));
}

#[no_mangle]
unsafe extern "C" fn MC_SCHEM_destroy_schem(ptr: *mut CSchematic) {
    (&mut *ptr).release();
}