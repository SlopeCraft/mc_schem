/*
mc_schem is a rust library to generate, load, manipulate and save minecraft schematic files.
Copyright (C) 2024  joseph

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::collections::{BTreeMap, HashMap};
use ndarray::{Array3};
use crate::block::Block;
use crate::error::Error;

/// Sky light and block light
#[derive(Debug, Copy, Clone)]
pub struct Light(u8);

/// An entity in MC, like zombie, minecart, etc.
#[derive(Debug, Clone)]
pub struct Entity {
    /// nbt tags of entity
    pub tags: HashMap<String, fastnbt::Value>,
    /// Position in double precision float
    pub position: [f64; 3],
    /// Position in integer
    pub block_pos: [i32; 3],
}

/// Block entity(also known as tile entity) in MC, like chest, furnace, etc.
#[derive(Debug, Clone)]
pub struct BlockEntity {
    /// nbt tags of block entity
    pub tags: HashMap<String, fastnbt::Value>,
}


#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum PendingTickInfo {
    Fluid { id: String },
    Block { id: String },
}

/// A tick waiting to be processed
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub struct PendingTick {
    pub priority: i32,
    pub sub_tick: i64,
    pub time: i32,
    pub info: PendingTickInfo,
}

pub trait HasPalette {
    fn palette(&self) -> &[Block];

    /// Find the block index of a block in palette
    fn find_in_palette(&self, block: &Block) -> Option<u16> {
        for (idx, blk) in self.palette().iter().enumerate() {
            if blk == block {
                return Some(idx as u16);
            }
        }
        return None;
    }


    /// Returns the block index of air in this region
    fn block_index_of_air(&self) -> Option<u16> {
        for (idx, blk) in self.palette().iter().enumerate() {
            if blk.is_air() {
                return Some(idx as u16);
            }
        }
        return None;
    }

    /// Returns the block index of structure void in this region
    fn block_index_of_structure_void(&self) -> Option<u16> {
        for (idx, blk) in self.palette().iter().enumerate() {
            if blk.is_structure_void() {
                return Some(idx as u16);
            }
        }
        return None;
    }
}


/// Part of a Minecraft world
pub trait WorldSlice {
    /// Shape in x, y, z
    fn shape(&self) -> [i32; 3];
    /// If `r_pos` is inside the region
    fn contains_coord(&self, r_pos: [i32; 3]) -> bool {
        for dim in 0..3 {
            if r_pos[dim] >= 0 && r_pos[dim] < self.shape()[dim] {
                continue;
            }
            return false;
        }
        return true;
    }
    /// Returns the volume
    fn volume(&self) -> u64 {
        return self.shape()[0] as u64 * self.shape()[1] as u64 * self.shape()[2] as u64;
    }
    ///Returns the count of blocks in region. Air will be counted if `include_air` is true, structure
    /// void is never counted.
    fn total_blocks(&self, include_air: bool) -> u64;
    /// Returns detailed block infos at `r_pos`, including block index, block, block entity and pending tick.
    /// Returns `None` if the block is outside the region
    fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, &[PendingTick])> {
        return Some((self.block_index_at(r_pos)?,
                     self.block_at(r_pos)?,
                     self.block_entity_at(r_pos),
                     self.pending_tick_at(r_pos),
        ));
    }
    /// Get block index at `r_pos`, returns `None` if the block is outside the region
    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16>;
    /// Get block at `r_pos`, returns `None` if the block is outside the region
    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block>;
    /// Get block entity at `r_pos`
    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity>;
    /// Get pending tick at `r_pos`
    fn pending_tick_at(&self, r_pos: [i32; 3]) -> &[PendingTick];
}

pub trait HasOffset {
    /// Offset of this region
    fn offset(&self) -> [i32; 3];
}


#[derive(Debug, Clone)]
pub struct Sparse3DArray {
    elements: BTreeMap<usize, u16>,
    shape: [usize; 3],
}

impl Default for Sparse3DArray {
    fn default() -> Self {
        Sparse3DArray {
            elements: BTreeMap::new(),
            shape: [0, 0, 0],
        }
    }
}

impl Sparse3DArray {
    pub fn new() -> Sparse3DArray {
        Sparse3DArray::default()
    }

    pub fn coordinate_3d_to_1d(coord: &[usize; 3], shape: &[usize; 3]) -> usize {
        let [sy, sz, sx] = shape;
        let [y, z, x] = coord;
        debug_assert!(*sy > 0 && *sz > 0 && *sx > 0);

        x + z * sx + y * sx * sz
    }

    pub fn coordinate_1d_to_3d(idx: usize, shape: &[usize; 3]) -> [usize; 3] {
        let [sy, sz, sx] = shape;
        debug_assert!(*sy > 0 && *sz > 0 && *sx > 0);
        let y = idx / (sx * sz);
        let rest = idx % (sx * sz);
        let z = rest / sx;
        let x = rest % sx;
        debug_assert!(x < *sx);
        debug_assert!(y < *sy);
        debug_assert!(z < *sz);
        [y, z, x]
    }
    pub fn reshape(&mut self, shape_new: &[usize; 3]) {
        self.elements.clear();
        self.shape = *shape_new;
    }

    pub fn zeros(shape: &[usize; 3]) -> Sparse3DArray {
        let mut ret = Self::new();
        ret.reshape(shape);
        ret
    }

    pub fn index_1d_max(&self) -> usize {
        self.shape[0] * self.shape[1] * self.shape[2]
    }
    pub fn get_1d(&self, idx: usize) -> u16 {
        *self.elements.get(&idx).unwrap_or(&0)
    }
    pub fn erase_1d(&mut self, idx: usize) -> Option<u16> {
        self.elements.remove(&idx)
    }
    pub fn set_1d(&mut self, idx: usize, value: u16) {
        if value == 0 {
            self.erase_1d(idx);
        } else {
            self.elements.insert(idx, value);
        }
    }

    pub fn get_3d(&self, pos: &[usize; 3]) -> u16 {
        self.get_1d(Self::coordinate_3d_to_1d(pos, &self.shape))
    }

    pub fn set_3d(&mut self, pos: &[usize; 3], value: u16) {
        self.set_1d(Self::coordinate_3d_to_1d(pos, &self.shape), value);
    }

    pub fn erase_3d(&mut self, pos: &[usize; 3]) -> Option<u16> {
        self.erase_1d(Self::coordinate_3d_to_1d(pos, &self.shape))
    }

    pub fn num_non_zero(&self) -> usize {
        self.elements.len()
    }

    /// Visit all non-air blocks
    pub fn visit_non_zero<F: FnMut(usize, &[usize; 3], u16)>(&self, func: &mut F) {
        for (idx_1d, value) in &self.elements {
            let idx_3d = Self::coordinate_1d_to_3d(*idx_1d, &self.shape);
            func(*idx_1d, &idx_3d, *value);
        }
    }
    /// Visit all non-air blocks with mutable
    pub fn visit_non_zero_mut<F: FnMut(usize, &[usize; 3], &mut u16)>(&mut self, func: &mut F) {
        for (idx_1d, value) in &mut self.elements {
            let idx_3d = Self::coordinate_1d_to_3d(*idx_1d, &self.shape);
            func(*idx_1d, &idx_3d, value);
        }
    }
    /// Visit all blocks including blank
    pub fn visit_dense<F: FnMut(usize, &[usize; 3], u16)>(&self, func: &mut F) {
        let mut previous_idx_1d = 0usize;
        for (idx_1d_nz, value) in &self.elements {
            for idx_1d in previous_idx_1d..*idx_1d_nz {
                let idx_3d = Self::coordinate_1d_to_3d(idx_1d, &self.shape);
                func(idx_1d, &idx_3d, 0);
            }
            let idx_3d = Self::coordinate_1d_to_3d(*idx_1d_nz, &self.shape);
            func(*idx_1d_nz, &idx_3d, *value);
            previous_idx_1d = *idx_1d_nz;
        }

        for idx_1d in previous_idx_1d..self.index_1d_max() {
            let idx_3d = Self::coordinate_1d_to_3d(idx_1d, &self.shape);
            func(idx_1d, &idx_3d, 0);
        }
    }

    pub fn into_dense(&self) -> Array3<u16> {
        let mut ret = Array3::zeros(self.shape);
        self.visit_non_zero(
            &mut |_, pos, val| { ret[*pos] = val; }
        );

        ret
    }
}

#[derive(Debug, Clone)]
pub enum Array3DVariant {
    Dense(Array3<u16>),
    Sparse(Sparse3DArray),
}

impl Array3DVariant {
    pub fn is_sparse(&self) -> bool {
        if let Array3DVariant::Dense(_) = self {
            return false;
        }
        true
    }

    pub fn is_dense(&self) -> bool {
        !self.is_sparse()
    }

    pub fn shape(&self) -> [usize; 3] {
        match self {
            Array3DVariant::Dense(arr) => {
                let sh = arr.shape();
                debug_assert!(sh.len() == 3);
                [sh[0], sh[1], sh[2]]
            },
            Array3DVariant::Sparse(arr) => {
                arr.shape.clone()
            }
        }
    }

    pub fn zeros(shape: &[usize; 3], dense: bool) -> Array3DVariant {
        if dense {
            return Array3DVariant::Dense(Array3::zeros(*shape));
        }
        Array3DVariant::Sparse(Sparse3DArray::zeros(shape))
    }

    pub fn reshape(&mut self, new_shape: &[usize; 3]) {
        match self {
            Array3DVariant::Dense(arr) => {
                *arr = Array3::zeros(*new_shape);
            },
            Array3DVariant::Sparse(arr) => {
                arr.reshape(new_shape);
            },
        }
    }

    pub fn get_3d(&self, pos: &[usize; 3]) -> u16 {
        match self {
            Array3DVariant::Dense(arr) => arr[*pos],
            Array3DVariant::Sparse(arr) => arr.get_3d(pos)
        }
    }

    pub fn set_3d(&mut self, pos: &[usize; 3], value: u16) {
        match self {
            Array3DVariant::Dense(arr) => arr[*pos] = value,
            Array3DVariant::Sparse(arr) => arr.set_3d(pos, value)
        }
    }

    pub fn visit_non_zero<F: FnMut(usize, &[usize; 3], u16)>(&self, func: &mut F) {
        match self {
            Array3DVariant::Dense(arr) => {
                for idx_1d in 0..arr.len() {
                    let pos = Sparse3DArray::coordinate_1d_to_3d(idx_1d, &self.shape());
                    func(idx_1d, &pos, arr[pos]);
                }
            },
            Array3DVariant::Sparse(arr) => {
                arr.visit_non_zero(func);
            },
        }
    }

    pub fn visit_non_zero_mut<F: FnMut(usize, &[usize; 3], &mut u16)>(&mut self, func: &mut F) {
        let shape = self.shape();
        match self {
            Array3DVariant::Dense(arr) => {
                for idx_1d in 0..arr.len() {
                    let pos = Sparse3DArray::coordinate_1d_to_3d(idx_1d, &shape);
                    func(idx_1d, &pos, &mut arr[pos]);
                }
            },
            Array3DVariant::Sparse(arr) => {
                arr.visit_non_zero_mut(func);
            },
        }
    }

    pub fn fill(&mut self, value: u16) {
        match self {
            Array3DVariant::Dense(arr) => {
                arr.fill(value);
                return;
            },
            Array3DVariant::Sparse(arr) => {
                if value == 0 {
                    arr.elements.clear();
                    return;
                }
                // Fill with non-zero
            }
        }
        let shape = self.shape();
        let mut new_arr = Array3::zeros(shape);
        new_arr.fill(value);
        *self = Array3DVariant::Dense(new_arr);
    }

    pub fn visit_dense<F: FnMut(usize, &[usize; 3], u16)>(&self, func: &mut F) {
        match self {
            Array3DVariant::Dense(arr) => {
                for idx_1d in 0..arr.len() {
                    let pos = Sparse3DArray::coordinate_1d_to_3d(idx_1d, &self.shape());
                    func(idx_1d, &pos, arr[pos]);
                }
            },
            Array3DVariant::Sparse(arr) => {
                arr.visit_dense(func);
            },
        }
    }

    pub fn convert_to_dense(&mut self) {
        if let Array3DVariant::Dense(_) = self {
            return;
        }
        if let Array3DVariant::Sparse(arr) = self {
            *self = Array3DVariant::Dense(arr.into_dense());
        }
    }

    pub fn convert_to_sparse(&mut self) {
        if let Array3DVariant::Sparse(_) = self {
            return;
        }
        let shape = self.shape();
        if let Array3DVariant::Dense(arr_old) = self {
            let mut arr = Sparse3DArray::zeros(&shape);
            let mut idx = 0usize;
            for val in arr_old.iter() {
                if *val != 0 {
                    arr.set_1d(idx, *val);
                }
                idx += 1;
            }

            *self = Array3DVariant::Sparse(arr);
        }
    }
}

/// Region is a 3d area in Minecraft, containing blocks and entities. \
/// Litematica files can have multiple regions, but vanilla structure, world edit schematics can have only one. \
/// Blocks in a region are stored as continuous 3d index array. A palette(Vec of blocks) records all
/// kinds of blocks in this region, so each block can be represented by an index(u16). Indices of
/// every block are stored in 3d array, indexed by y, z, x. YZX is applied because all schematic
/// formats store blocks in this order, by following this custom, our library can have better performance in loading and saving.
#[derive(Debug, Clone)]
pub struct Region {
    /// Name of this region, only useful in litematica
    pub name: String,
    /// Array of block indices, stored in y,z,x
    pub array_yzx: Array3DVariant,
    /// All kinds of blocks
    pub palette: Vec<Block>,
    /// All block entities. The key is position (xyz)
    pub block_entities: HashMap<[i32; 3], BlockEntity>,
    /// All pending ticks. The key is position (xyz)
    pub pending_ticks: HashMap<[i32; 3], Vec<PendingTick>>,
    /// All entities
    pub entities: Vec<Entity>,
    /// Offset of this region
    pub offset: [i32; 3],


    //pub array_number_id_damage: Option<Array3<(u8, u8)>>
}

impl Default for Light {
    fn default() -> Self {
        return Self(0xFF);
    }
}

impl Light {
    pub fn new(sky_light: u8, block_light: u8) -> Self {
        return Self(sky_light << 4 | block_light);
    }

    pub fn sky_light(&self) -> u8 {
        return (self.0 & 0xF0) >> 4;
    }
    pub fn block_light(&self) -> u8 {
        return self.0 & 0x0F;
    }
}

impl Entity {
    pub fn new() -> Entity {
        return Entity {
            tags: HashMap::new(),
            position: [0.0, 0.0, 0.0],
            block_pos: [0, 0, 0],
        };
    }

    /// Add adder to position and block_pos
    pub fn pos_shift(&mut self, adder: [i32; 3]) {
        for dim in 0..3 {
            self.block_pos[dim] += adder[dim];
            self.position[dim] += adder[dim] as f64;
        }
    }
}

impl BlockEntity {
    pub fn new() -> BlockEntity {
        return BlockEntity {
            tags: HashMap::new(),
        };
    }
}


impl PendingTickInfo {
    pub fn default() -> PendingTickInfo {
        return PendingTickInfo::Block { id: "".to_string() };
    }
}

impl HasPalette for Region {
    fn palette(&self) -> &[Block] {
        &self.palette
    }
}

impl HasOffset for Region {
    fn offset(&self) -> [i32; 3] {
        self.offset
    }
}

impl WorldSlice for Region {

    /// Shape in x, y, z
    fn shape(&self) -> [i32; 3] {
        let shape = self.array_yzx.shape();
        if shape.len() != 3 {
            panic!("Invalid array dimensions: should be 3 but now it is {}", shape.len());
        }
        return Self::pos_yzx_to_xyz(&[shape[0] as i32, shape[1] as i32, shape[2] as i32]);
    }
    ///Returns the count of blocks in region. Air will be counted if `include_air` is true, structure
    /// void is never counted.
    fn total_blocks(&self, include_air: bool) -> u64 {
        let mut counter = 0;
        let air_idx_opt = self.block_index_of_air();
        let sv_idx_opt = self.block_index_of_structure_void();

        self.array_yzx.visit_dense(
            &mut |_idx_1d: usize, _pos: &[usize; 3], blk_id: u16| {
                if let Some(air_idx) = air_idx_opt {
                    if blk_id == air_idx {
                        if include_air {
                            counter += 1;
                        }
                        return;
                    }
                }

                if let Some(sv_idx) = sv_idx_opt {
                    if blk_id == sv_idx {
                        counter += 1;
                        return;
                    }
                }

                counter += 1;
            }
        );
        counter
    }

    /// Returns detailed block infos at `r_pos`, including block index, block, block entity and pending tick.
    /// Returns `None` if the block is outside the region
    fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, &[PendingTick])> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some((pid, &self.palette[pid as usize],
                  self.block_entities.get(&r_pos),
                  self.pending_tick_at(r_pos)))
        } else {
            None
        };
    }
    /// Get block index at `r_pos`, returns `None` if the block is outside the region
    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        if !self.contains_coord(r_pos) {
            return None;
        }

        let x = r_pos[0] as usize;
        let y = r_pos[1] as usize;
        let z = r_pos[2] as usize;

        let pid = self.array_yzx.get_3d(&[y, z, x]) as usize;
        Some(pid as u16)
    }
    /// Get block at `r_pos`, returns `None` if the block is outside the region
    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some(&self.palette[pid as usize])
        } else {
            None
        };
    }
    /// Get block entity at `r_pos`
    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity> {
        return self.block_entities.get(&r_pos);
    }

    /// Get pending tick at `r_pos`
    fn pending_tick_at(&self, r_pos: [i32; 3]) -> &[PendingTick] {
        if let Some(pts) = self.pending_ticks.get(&r_pos) {
            return &pts;
        }
        &[]
    }
}


#[allow(dead_code)]
impl Region {
    /// Convert pos from xyz to yzx
    pub fn pos_xyz_to_yzx<T>(pos: &[T; 3]) -> [T; 3]
        where T: Copy {
        [pos[1], pos[2], pos[0]]
    }

    /// Convert pos from yzx to xyz
    pub fn pos_yzx_to_xyz<T>(yzx: &[T; 3]) -> [T; 3]
        where T: Copy {
        [yzx[2], yzx[0], yzx[1]]
    }

    /// Create a new region with size \[1,1,1\], filled with air
    pub fn new() -> Region {
        Self::with_shape([1, 1, 1])
    }

    pub fn with_shape(shape_xyz: [i32; 3]) -> Region {
        let shape_yzx = [shape_xyz[1] as usize, shape_xyz[2] as usize, shape_xyz[0] as usize];
        //let shape_zx = [shape_xyz[2], shape_xyz[1]];
        let mut result = Region {
            name: String::from("NewRegion"),
            array_yzx: Array3DVariant::zeros(&shape_yzx, true),
            palette: Vec::new(),
            block_entities: HashMap::new(),
            pending_ticks: HashMap::new(),
            entities: Vec::new(),
            offset: [0, 0, 0],
        };
        result.find_or_append_to_palette(&Block::air());
        return result;
    }

    /// Convert pos in `[i32;3]` to `[usize;3]`
    pub fn i32_to_usize(pos: &[i32; 3]) -> [usize; 3] {
        let x = pos[0] as usize;
        let y = pos[1] as usize;
        let z = pos[2] as usize;

        [x, y, z]
    }

    /// Set block as assigned position. `r_pos` is a relative pos in xyz. \
    /// If there's block in palette same as `block`, the palette won't change, otherwise `block`
    /// will be cloned and pushed to palette. \
    /// This function returns `Err(())` if block palette exceeds 65535, which seldom happens.
    pub fn set_block(&mut self, r_pos: [i32; 3], block: &Block) -> Result<(), ()> {
        if !self.contains_coord(r_pos) {
            return Err(());
        }
        let mut blkid = self.palette.len();
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk == block {
                blkid = idx;
                break;
            }
        }
        if blkid >= self.palette.len() {
            self.palette.push(block.clone());
        }
        if blkid >= 65536 {
            return Err(());
        }
        let blkid = blkid as u16;

        let pos_usize = Self::i32_to_usize(&r_pos);
        self.array_yzx.set_3d(&Self::pos_xyz_to_yzx(&pos_usize), blkid);

        return Ok(());
    }

    /// Set block index as `r_pos`. If `block_id` >= length of palette, returns `Err(())`
    pub fn set_block_id(&mut self, r_pos: [i32; 3], block_id: u16) -> Result<(), ()> {
        if !self.contains_coord(r_pos) {
            return Err(());
        }
        if block_id as usize >= self.palette.len() {
            return Err(());
        }
        let pos_usize = Self::i32_to_usize(&r_pos);
        self.array_yzx.set_3d(&Self::pos_xyz_to_yzx(&pos_usize), block_id);
        return Ok(());
    }

    /// Reshape the region and fill `array_yzx` with 0
    pub fn reshape(&mut self, shape_xyz: &[i32; 3]) {
        let mut usz: [usize; 3] = [0, 0, 0];
        for idx in 0..3 {
            let sz = shape_xyz[idx];
            if sz < 0 {
                panic!("Try resizing with negative size [{},{},{}]", shape_xyz[0], shape_xyz[1], shape_xyz[2]);
            }
            usz[idx] = sz as usize;
        }
        let shape_yzx = Self::pos_xyz_to_yzx(&usz);
        self.array_yzx.reshape(&shape_yzx);
        //let shape_zx = [shape_xyz[2], shape_xyz[1]];
        // self.sky_block_light = Array3::default(shape_yzx);
        // self.sky_block_light.fill(Light::default());
        // self.biome = Array2::default(shape_zx);
    }

    /// Shape in y, z, x
    pub fn shape_yzx(&self) -> [i32; 3] {
        let shape = self.array_yzx.shape();
        if shape.len() != 3 {
            panic!("Invalid array dimensions: should be 3 but now it is {}", shape.len());
        }
        [shape[0] as i32, shape[1] as i32, shape[2] as i32]
    }

    /// Convert global position to relative position. `r_pos` = `g_pos` - `self.offset`
    pub fn global_pos_to_relative_pos(&self, g_pos: [i32; 3]) -> [i32; 3] {
        [
            g_pos[0] - self.offset[0],
            g_pos[1] - self.offset[1],
            g_pos[2] - self.offset[2],
        ]
    }

    /// Convert relative position to global position. `g_pos` = `r_pos` + `self.offset`
    pub fn relative_pos_to_global_pos(&self, r_pos: [i32; 3]) -> [i32; 3] {
        [
            r_pos[0] + self.offset[0],
            r_pos[1] + self.offset[1],
            r_pos[2] + self.offset[2],
        ]
    }
    /// Remove non-existing blocks from palette. Returns error if there is any block index that is
    /// equal or greater than length of palette
    pub fn shrink_palette(&mut self) -> Result<(), Error> {
        let mut block_counter: Vec<u64> = Vec::new();
        block_counter.resize(self.palette.len(), 0);

        let mut ret: Result<(), Error> = Ok(());
        self.array_yzx.visit_dense(
            &mut |_, pos, blk_idx| {
                if let Err(_) = &ret {
                    return;
                }
                if blk_idx as usize >= self.palette.len()
                {
                    let [y, z, x] = *pos;
                    ret = Err(Error::BlockIndexOutOfRangeWriting {
                        r_pos: [x as i32, y as i32, z as i32],
                        block_index: blk_idx,
                        max_index: self.palette.len() as u16 - 1,
                    });
                }
                block_counter[blk_idx as usize] += 1;
            }
        );
        if ret.is_err() {
            return ret;
        }


        let mut id_map: Vec<u16> = Vec::new();
        id_map.resize(self.palette.len(), 65535);
        {
            let mut counter: u16 = 0;
            for id in 0..self.palette.len() {
                if block_counter[id] <= 0 {
                    continue;
                }
                id_map[id] = counter;
                counter += 1;
            }
            for id in (0..block_counter.len()).rev() {
                if block_counter[id] <= 0 {
                    self.palette.remove(id);
                }
            }
        }

        self.array_yzx.visit_non_zero_mut(
            &mut |_, _, blkid| {
                let new_id = id_map[*blkid as usize];
                assert!((new_id as usize) < self.palette.len());
                *blkid = new_id;
            }
        );

        Ok(())
    }

    /// Find the block index of a block in palette
    pub fn find_in_palette(&self, block: &Block) -> Option<u16> {
        for (idx, blk) in self.palette.iter().enumerate() {
            if blk == block {
                return Some(idx as u16);
            }
        }
        return None;
    }

    /// Find the block in palette, if not exist, append it to the palette.
    pub fn find_or_append_to_palette(&mut self, block: &Block) -> u16 {
        return match self.find_in_palette(block) {
            Some(idx) => idx,
            None => {
                self.palette.push(block.clone());
                (self.palette.len() - 1) as u16
            }
        }
    }
    /// Fill the region with block
    pub fn fill_with(&mut self, block: &Block) {
        let blk_id = self.find_or_append_to_palette(block);
        self.array_yzx.fill(blk_id);
    }

    /// Set block entity at `r_pos`
    pub fn set_block_entity_at(&mut self, r_pos: [i32; 3], be: BlockEntity) -> Option<BlockEntity> {
        return self.block_entities.insert(r_pos, be);
    }
    /// Set pending tick at `r_pos`
    pub fn set_pending_tick_at(&mut self, r_pos: [i32; 3], value: Vec<PendingTick>) -> Option<Vec<PendingTick>> {
        return self.pending_ticks.insert(r_pos, value);
    }

    /// Returns detailed block infos at `r_pos`, including block index, block, block entity(mutable) and pending tick(mutable).
    /// Returns `None` if the block is outside the region
    pub fn block_info_at_mut(&mut self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&mut BlockEntity>, &mut [PendingTick])> {
        return if let Some(pid) = self.block_index_at(r_pos) {
            Some((pid, &self.palette[pid as usize],
                  self.block_entities.get_mut(&r_pos),
                  if let Some(pts) = self.pending_ticks.get_mut(&r_pos) { pts.as_mut_slice() } else { &mut [] }
            ))
        } else {
            None
        };
    }
    /// Get mutable block entity at `r_pos`
    pub fn block_entity_at_mut(&mut self, r_pos: [i32; 3]) -> Option<&mut BlockEntity> {
        return self.block_entities.get_mut(&r_pos);
    }

    /// Get mutable pending tick at `r_pos`
    pub fn pending_tick_at_mut(&mut self, r_pos: [i32; 3]) -> &mut [PendingTick] {
        return if let Some(pts) = self.pending_ticks.get_mut(&r_pos) {
            pts.as_mut_slice()
        } else {
            &mut []
        };
    }

    pub fn convert_to_dense(&mut self) {
        self.array_yzx.convert_to_dense();
    }

    pub fn is_dense(&self) -> bool {
        self.array_yzx.is_dense()
    }

    pub fn is_sparse(&self) -> bool {
        self.array_yzx.is_sparse()
    }

    pub fn convert_to_sparse(&mut self) {
        self.array_yzx.convert_to_sparse();
    }
}