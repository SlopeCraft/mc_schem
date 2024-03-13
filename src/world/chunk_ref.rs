use crate::block::Block;
use crate::region::{BlockEntity, HasOffset, PendingTick, WorldSlice};
use crate::world::{ChunkRefAbsolutePos, ChunkRefRelativePos, SubChunk};

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

    fn block_at(&self, r_pos: [i32; 3]) -> Option<&Block> {
        if self.contains_coord(r_pos) {
            let (_, sect, pos) = self.to_sub_chunk_r_pos(r_pos);
            return sect.block_at(pos);
        }
        return None;
    }

    fn block_entity_at(&self, r_pos: [i32; 3]) -> Option<&BlockEntity> {
        if self.contains_coord(r_pos) {
            return self.chunk.block_entities.get(&self.to_absolute_pos(r_pos));
        }
        return None;
    }

    fn pending_tick_at(&self, r_pos: [i32; 3]) -> Option<&PendingTick> {
        if self.contains_coord(r_pos) {
            return self.chunk.pending_ticks.get(&self.to_absolute_pos(r_pos));
        }
        return None;
    }
}