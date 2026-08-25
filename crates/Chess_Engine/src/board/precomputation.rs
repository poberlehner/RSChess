use std::cmp::min;

use crate::board::{Board, r#move::KNIGHT_JUMP_OFFSETS};

impl Board {
    pub(super) fn precompute_distance_to_edge_data(&mut self) {
        for (i, distance_to_edge) in self.distance_to_edge.iter_mut().enumerate() {
            let x = i % 8;
            let y = i / 8;

            let north_num = 7 - y;
            let east_num = 7 - x;
            let south_num = y;
            let west_num = x;

            *distance_to_edge = [
                north_num as u8,
                east_num as u8,
                south_num as u8,
                west_num as u8,
                min(north_num, east_num) as u8,
                min(east_num, south_num) as u8,
                min(south_num, west_num) as u8,
                min(west_num, north_num) as u8,
            ];
        }
    }

    pub(super) fn precompute_knight_jump_legality(&mut self) {
        self.knight_jump_legality = [[true; 8]; 64];

        for (i, knight_jump_legality) in self.knight_jump_legality.iter_mut().enumerate() {
            for (dir, offset) in KNIGHT_JUMP_OFFSETS.iter().enumerate() {
                let x = i as i32 % 8;
                let y = i as i32 / 8;

                let new_x = (i as i32 + offset) % 8;
                let new_y = (i as i32 + offset) / 8;

                let total_diff = x.abs_diff(new_x) + y.abs_diff(new_y);

                if total_diff != 3 {
                    knight_jump_legality[dir] = false;
                }

                if i as i32 + offset < 0 || i as i32 + offset > 63 {
                    knight_jump_legality[dir] = false;
                }
            }
        }
    }
}