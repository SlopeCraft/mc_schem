use std::collections::HashMap;
use fastnbt::Value;
use crate::error::Error;
use crate::world::Chunk;

impl Chunk {
    pub fn from_nbt(mut nbt: HashMap<String, Value>) -> Result<Chunk, Error> {
        !todo!();
    }
}