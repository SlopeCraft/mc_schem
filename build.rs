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

fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        // 0x0409 = en-US, 0x0804 = zh-CN.
        res.set_language(0x0409)
            .set("CompanyName", "SlopeCraft")
            .set("ProductName", "mc_schem")
            .set("InternalName", "mc_schem")
            .set(
                "FileDescription",
                "Utility and library for Minecraft world and schematics",
            )
            .set(
                "LegalCopyright",
                "Copyright (C) 2026 ToKiNoBug. Licensed under GPL-3.0-only",
            )
            .set("Comments", "https://github.com/SlopeCraft/mc_schem");

        if let Err(e) = res.compile() {
            // Missing version metadata is cosmetic, so do not fail the build
            // over it (a GNU target needs windres.exe and ar.exe on PATH).
            println!("cargo:warning=failed to embed the Windows version resource: {e}");
        }
    }
}
