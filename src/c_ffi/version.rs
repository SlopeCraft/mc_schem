#[no_mangle]
pub extern "C" fn mc_schem_version() -> u32 {
    0x01_03_02_00
}

#[no_mangle]
pub extern "C" fn mc_schem_version_anchor_v1_3_0() -> u32 {
    mc_schem_version()
}