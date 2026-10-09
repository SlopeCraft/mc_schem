use crate::c_ffi::{rust_reader, rust_string_receiver, rust_writer};
use fastnbt::Value;
use std::collections::HashMap;
use std::ptr::null_mut;

//nbt_hashmap* mc_schem_create_nbt_hashmap()
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap() -> *mut HashMap<String, Value> {
    Box::into_raw(Box::from(HashMap::new()))
}

// [[nodiscard]] size_t mc_schem_nbt_hashmap_get_size(const nbt_hashmap*);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_nbt_hashmap_get_size(
    ptr: *const HashMap<String, Value>,
) -> usize {
    ptr.as_ref_unchecked().len()
}

//[[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_binary( const uint8_t* buffer, size_t bytes, const rust_string_receiver* error_message_receiver);#[no_mangle]
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap_from_binary(
    buffer: *const u8,
    len: usize,
    error_msg_receiver: *const rust_string_receiver,
) -> *mut HashMap<String, Value> {
    match fastnbt::from_bytes::<HashMap<String, Value>>(std::slice::from_raw_parts(buffer, len)) {
        Err(e) => {
            error_msg_receiver
                .as_ref_unchecked()
                .receive(&e.to_string());
            null_mut()
        }
        Ok(nbt) => Box::into_raw(Box::from(nbt)),
    }
}

// [[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_binary_stream(istream_wrapper* src, const rust_string_receiver* error_message_receiver);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_create_nbt_hashmap_from_binary_stream(
    src: *mut rust_reader,
    error_msg_receiver: *const rust_string_receiver,
) -> *mut HashMap<String, Value> {
    let nbt: Result<HashMap<String, Value>, _> = fastnbt::from_reader(src.as_mut_unchecked());
    match nbt {
        Err(e) => {
            error_msg_receiver
                .as_ref_unchecked()
                .receive(&e.to_string());
            null_mut()
        }
        Ok(nbt) => Box::into_raw(Box::from(nbt)),
    }
}
// bool mc_schem_nbt_hashmap_dump_to_binary_stream(const nbt_hashmap*, ostream_wrapper* dest, const rust_string_receiver* error_message_receiver);
#[no_mangle]
pub unsafe extern "C" fn mc_schem_nbt_hashmap_dump_to_binary_stream(
    nbt: *const HashMap<String, Value>,
    os: *mut rust_writer,
    error_msg_receiver: *const rust_string_receiver,
) -> bool {
    let result = fastnbt::to_writer(os.as_mut_unchecked(), nbt.as_ref_unchecked());
    if let Err(e) = &result {
        error_msg_receiver
            .as_ref_unchecked()
            .receive(&e.to_string())
    }
    result.is_ok()
}
