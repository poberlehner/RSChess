#![allow(clippy::unwrap_used, clippy::expect_used)]
use chess_engine::board::utilities::TimeControl;
use criterion::{criterion_group, criterion_main, Criterion};
use std::env;
use std::fs::exists;
use std::path::PathBuf;
use chess_engine::board::Board;

pub fn move_generation_recursive_bench(c: &mut Criterion) {
    let current_directory = env::current_dir();
    let mut current_directory = current_directory.unwrap();

    while !(exists(current_directory.join(PathBuf::from("assets"))).is_ok() && exists(current_directory.join(PathBuf::from("assets"))).unwrap())  {

        current_directory.pop();
    }

    env::set_current_dir(current_directory).expect("Failed to set current working directory");

    let mut board: Board = Board::new();

    board.start_game(TimeControl::Unlimited);

    #[allow(non_snake_case)]
    let DEPTH = 3;

    let mut group = c.benchmark_group("Move generation");

    group.noise_threshold(0.02);
    group.confidence_level(0.90);
    group.sample_size(20000);
    group.measurement_time(core::time::Duration::from_secs(9));
    group.bench_function("Move generation", |b| b.iter(|| recursive_move_generation_bench(&mut board, DEPTH)));
}

fn recursive_move_generation_bench(board: &mut Board, depth: u32) -> u64 {
    let legal_moves = board.get_legal_moves().clone().expect("Cloning the legal move vector failed (in recursive function)");

    if depth == 1 {
        return legal_moves.len() as u64;
    }

    let mut num_positions = 0;

    for legal_move in legal_moves {
        if !board.make_move(legal_move, true) {
            println!("Executing move {}{} failed (depth: {})",
                Board::square_index_to_chars(legal_move.origin_square).iter().collect::<String>(),
                Board::square_index_to_chars(legal_move.destination_square).iter().collect::<String>(), depth);
        }

        let num_new_positions = recursive_move_generation_bench(board, depth - 1);
        num_positions += num_new_positions;

        board.undo_move(true);
    }

    num_positions
}

criterion_group!(benches, move_generation_recursive_bench);
criterion_main!(benches);