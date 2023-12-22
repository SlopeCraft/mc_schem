mod vanilla_structure {
    use std::io;
    use crate::schem::schem;
    use compress::zlib;
    use crate::schem::schem::Schematic;
    use nbt;

    pub enum VanillaStructureLoadError {
        NBTReadError(nbt::Error),
    }

    impl schem::Schematic {
        pub fn from_vanilla_structure(src: &mut dyn std::io::Read) -> Result<Schematic, VanillaStructureLoadError> {
            let loaded_opt: Result<nbt::Blob, nbt::Error> = nbt::from_gzip_reader(src);
            let loaded: nbt::Blob;
            match loaded_opt {
                Ok(nbt) => loaded = nbt,
                Err(err) => return Err(VanillaStructureLoadError::NBTReadError(err)),
            }

            let result = schem::Schematic::new();
        }
    }
}
