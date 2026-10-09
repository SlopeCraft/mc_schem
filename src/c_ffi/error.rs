/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2026 ToKiNoBug

 This program is free software: you can redistribute it and/or modify it under
 the terms of the GNU General Public License as published by the Free Software
 Foundation, either version 3 of the License, or (at your option) any later
 version.

 This program is distributed in the hope that it will be useful, but WITHOUT ANY
 WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 PARTICULAR PURPOSE.  See the GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
