use std::ops::Range;
use crate::block::Block;
use crate::region::{BlockEntity, HasOffset, PendingTick, WorldSlice};
use crate::world::{AbsolutePosIndexed, ChunkRefAbsolutePos, ChunkRefRelativePos, SubChunk};

impl ChunkRefRelativePos<'_> {
    fn y_pos_to_section_number(&self, y_r: i32) -> i8 {
        let y_a = y_r + self.chunk.y_offset();
        return (y_a / 16) as i8;
    }

    fn to_sub_chunk_r_pos(&self, r_pos: [i32; 3]) -> (i8, &SubChunk, [i32; 3]) {
        let y_sect_num = self.y_pos_to_section_number(r_pos[1]);
        let sect = &self.chunk.sub_chunks[&y_sect_num];
        let y_offset = y_sect_num as i32 * 16;
        debug_assert!(r_pos[1] - y_offset >= 0);
        let pos = [r_pos[0], r_pos[1] - y_offset, r_pos[2]];
        debug_assert!(sect.contains_coord(pos));
        return (y_sect_num, sect, pos);
    }

    fn to_absolute_pos(&self, r_pos: [i32; 3]) -> [i32; 3] {
        let y_a = self.chunk.y_offset() + r_pos[1];
        let lb = self.chunk_pos.block_pos_lower_bound();
        let x_a = r_pos[0] + lb[0];
        let z_a = r_pos[2] + lb[1];
        return [x_a, y_a, z_a];
    }
}

impl HasOffset for ChunkRefRelativePos<'_> {
    fn offset(&self) -> [i32; 3] {
        return [0, self.chunk.y_offset(), 0];
    }
}

impl WorldSlice for ChunkRefRelativePos<'_> {
    fn shape(&self) -> [i32; 3] {
        return self.chunk.shape();
    }

    fn total_blocks(&self, include_air: bool) -> u64 {
        return self.chunk.total_blocks(include_air);
    }

    fn block_index_at(&self, r_pos: [i32; 3]) -> Option<u16> {
        if self.contains_coord(r_pos) {
            let (_, sect, pos) = self.to_sub_chunk_r_pos(r_pos);
            return sect.block_index_at(pos);
        }
        return None;
    }

    fn block_at(&self, r_pos: [i32; 3]) -> Option<&'_ Block> {
        if self.contains_coord(r_pos) {
            let (_, sect, pos) = self.to_sub_chunk_r_pos(r_pos);
            return sect.block_at(pos);
        }
        return None;
    }

    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&'_ BlockEntity> {
        if self.contains_coord(r_pos) {
            return self.chunk.block_entities.get(&self.to_absolute_pos(r_pos));
        }
        return None;
    }

    fn pending_tick_at(&self, r_pos: [i32; 3]) -> &'_ [PendingTick] {
        if self.contains_coord(r_pos) {
            return if let Some(pts) = self.chunk.pending_ticks.get(&self.to_absolute_pos(r_pos)) {
                pts
            } else {
                &[]
            }
        }
        return &[];
    }
}

impl<'s, 'chunk: 's> ChunkRefAbsolutePos<'chunk> {
    fn to_sub_chunk_r_pos(&'s self, a_pos: [i32; 3]) -> (i8, &'chunk SubChunk, [i32; 3]) {
        let sect_number = (a_pos[1] / 16) as i8;
        debug_assert!(self.chunk.sub_chunks.contains_key(&sect_number));
        let sub_chunk: &'chunk SubChunk = self.chunk.sub_chunks.get(&sect_number).unwrap();
        let o = self.offset();
        let r_pos = [a_pos[0] - o[0], a_pos[1] - sect_number as i32 * 16, a_pos[2] - o[2]];
        debug_assert!((0..16).contains(&r_pos[1]));
        return (sect_number, sub_chunk, r_pos);
    }
}

impl HasOffset for ChunkRefAbsolutePos<'_> {
    fn offset(&self) -> [i32; 3] {
        return [self.chunk_pos.block_pos_lower_bound()[0],
            self.chunk.y_offset(),
            self.chunk_pos.block_pos_lower_bound()[1]];
    }
}

impl<'s, 'chunk: 's> AbsolutePosIndexed<'s, 'chunk> for ChunkRefAbsolutePos<'chunk> {
    fn pos_range(&self) -> [Range<i32>; 3] {
        let o = self.offset();
        return [
            o[0]..(o[0] + 16),
            self.chunk.y_range(),
            o[2]..(o[2] + 16),
        ];
    }

    fn total_blocks(&self, include_air: bool) -> u64 {
        return self.chunk.total_blocks(include_air);
    }

    fn block_index_at(&self, a_pos: [i32; 3]) -> Option<u16> {
        if self.contains_coord(a_pos) {
            let (_, sect, r_pos) = self.to_sub_chunk_r_pos(a_pos);
            debug_assert!(sect.contains_coord(r_pos));
            return sect.block_index_at(r_pos);
        }
        return None;
    }

    fn block_at(&'s self, a_pos: [i32; 3]) -> Option<&'chunk Block> {
        if self.contains_coord(a_pos) {
            let (_, sect, r_pos) = self.to_sub_chunk_r_pos(a_pos);
            debug_assert!(sect.contains_coord(r_pos));
            return sect.block_at(r_pos);
        }
        return None;
    }

    fn block_entity_at(&self, a_pos: [i32; 3]) -> Option<&'chunk BlockEntity> {
        return self.chunk.block_entities.get(&a_pos);
    }

    fn pending_tick_at(&self, a_pos: [i32; 3]) -> &'chunk [PendingTick] {
        return if let Some(pts) = self.chunk.pending_ticks.get(&a_pos) {
            pts
        } else {
            &[]
        };
    }
}