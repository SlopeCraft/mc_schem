use crate::c_ffi::rust_string_receiver;
use crate::error::Error;

#[no_mangle]
pub unsafe extern "C" fn mc_schem_error_get_message(
    err: *const Error,
    receiver: *const rust_string_receiver,
) {
    if err.is_null() {
        (*receiver).receive("");
        return;
    }
    (*receiver).receive(&(*err).to_string());
}
