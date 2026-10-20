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

// The Windows version resource is attached per target kind: a build script never
// learns which target is being linked, so res/mc_schem.rc goes to the cdylib and
// res/schemtool.rc to the `schemtool` bin.
// `embed-resource` is only compiled on Windows, hence the `cfg`.

fn main() {
    #[cfg(windows)]
    {
        let macros = version_macros();

        report(embed_resource::compile_for_cdylib(
            "res/mc_schem.rc",
            &macros,
        ));
        report(embed_resource::compile_for(
            "res/schemtool.rc",
            &["schemtool"],
            &macros,
        ));
    }
}

/// Version numbers as rc.exe macro definitions, so that Cargo.toml stays the
/// only place where the version is written.
#[cfg(windows)]
fn version_macros() -> Vec<String> {
    let env = |name: &str| std::env::var(name).unwrap_or_else(|_| "0".to_owned());
    vec![
        format!("MC_SCHEM_VERSION_MAJOR={}", env("CARGO_PKG_VERSION_MAJOR")),
        format!("MC_SCHEM_VERSION_MINOR={}", env("CARGO_PKG_VERSION_MINOR")),
        format!("MC_SCHEM_VERSION_PATCH={}", env("CARGO_PKG_VERSION_PATCH")),
        format!("MC_SCHEM_VERSION_STRING=\"{}\"", env("CARGO_PKG_VERSION")),
    ]
}

/// Reports a missing or failed resource as a warning: version metadata is
/// cosmetic, and not every setup has a resource compiler (a GNU target needs
/// windres, an MSVC one the Windows SDK), so this never fails the build.
#[cfg(windows)]
fn report(result: embed_resource::CompilationResult) {
    match result {
        embed_resource::CompilationResult::Ok => {}
        other => println!("cargo:warning=Windows version resource not embedded: {other}"),
    }
}
