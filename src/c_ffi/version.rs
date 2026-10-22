#[no_mangle]
pub extern "C" fn mc_schem_version() -> u32 {
    0x01_03_00_00
}

#[no_mangle]
pub extern "C" fn mc_schem_v1_3_0_anchor() -> u32 {
    mc_schem_version()
}