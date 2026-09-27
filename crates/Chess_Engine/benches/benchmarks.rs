#![allow(clippy::unwrap_used, clippy::expect_used)]

use chess_engine::board::{piece::{Piece, PieceType}, utilities::TimeControl};
use criterion::{Bencher, Criterion, criterion_group, criterion_main};
use chess_engine::board::Board;

static MIDGAMEFEN: &str = "r1bq1rk1/pppp1pp1/2n2n1p/2b1p3/2B1P3/2NPBN2/PPP2PPP/R2Q1RK1 b - - 3 7";
static ENDGAMEFEN: &str = "5R2/p5p1/6P1/2r2Pk1/8/8/Pb2K3/8 b - - 4 43";

pub fn perft_benches(c: &mut Criterion) {
    let mut board: Board = Board::new();

    #[allow(non_snake_case)]
    let DEPTH = 2;

    let mut group = c.benchmark_group(format!("Perft Bench (depth {DEPTH})"));

    group.noise_threshold(0.02);
    group.confidence_level(0.90);
    group.measurement_time(core::time::Duration::from_secs(10));

    board.start_game(TimeControl::Unlimited);
    group.bench_function("startpos", |b| b.iter(|| recursive_move_generation_bench(&mut board, DEPTH)));

    board.load_fen(MIDGAMEFEN);
    board.start_game(TimeControl::Unlimited);

    group.bench_function("midgame", |b| b.iter(|| recursive_move_generation_bench(&mut board, DEPTH)));

    board.load_fen(ENDGAMEFEN);
    board.start_game(TimeControl::Unlimited);

    group.bench_function("endgame", |b| b.iter(|| recursive_move_generation_bench(&mut board, DEPTH)));
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

fn bitboard_generation_benches(c: &mut Criterion) {
    let mut board: Board = Board::new();

    let mut group = c.benchmark_group("Bitboard generation Bench");

    group.noise_threshold(0.02);
    group.confidence_level(0.90);
    group.measurement_time(core::time::Duration::from_secs(10));

    board.start_game(TimeControl::Unlimited);
    group.bench_function("startpos", |b| b.iter(|| bitboard_bench(&mut board)));

    board.load_fen(MIDGAMEFEN);
    board.start_game(TimeControl::Unlimited);

    group.bench_function("midgame", |b| b.iter(|| bitboard_bench(&mut board)));

    board.load_fen(ENDGAMEFEN);
    board.start_game(TimeControl::Unlimited);

    group.bench_function("endgame", |b| b.iter(|| bitboard_bench(&mut board)));
}

fn bitboard_bench(board: &mut Board) {
    board.generate_all_bitboards();
    board.invalidate_caches();
}

fn movegen_bench(c: &mut Criterion) {
    let mut board: Board = Board::new();

    let mut group = c.benchmark_group("Pure move generation Bench");

    group.noise_threshold(0.02);
    group.confidence_level(0.90);
    group.measurement_time(core::time::Duration::from_secs(10));

    board.start_game(TimeControl::Unlimited);
    board.invalidate_caches();

    group.bench_function("startpos", |b| b.iter(|| { board.get_legal_moves(); board.invalidate_caches(); }));

    board.load_fen(MIDGAMEFEN);
    board.start_game(TimeControl::Unlimited);
    board.invalidate_caches();

    group.bench_function("midgame", |b| b.iter(|| { board.get_legal_moves(); board.invalidate_caches(); }));

    board.load_fen(ENDGAMEFEN);
    board.start_game(TimeControl::Unlimited);
    board.invalidate_caches();

    group.bench_function("endgame", |b| b.iter(|| { board.get_legal_moves(); board.invalidate_caches(); }));

    group.finish();
}

fn piece_benches(c: &mut Criterion) {
    let pieces = [PieceType::King, PieceType::Queen, PieceType::Rook, PieceType::Knight, PieceType::Bishop, PieceType::Pawn];
    let piece_names = ["King", "Queen", "Rook", "Knight", "Bishop", "Pawn"];
    let mut board: Board = Board::new();

    let mut group = c.benchmark_group("Piece full movegen benchmarks");

    group.sample_size(500);
    group.confidence_level(0.90);
    group.noise_threshold(0.02);

    for i in 0..6 {
        group.bench_function(format!("{}", piece_names[i]), |b| bench_piece(b, &mut board, pieces[i]));
    }
}

fn bench_piece(b: &mut Bencher, board: &mut Board, piece: PieceType) {
    board.load_fen(&format!("8/8/8/4{}3/8/8/8/8 w - - 0 1", Board::piece_to_char(Piece {piece_type: piece, white_piece: true})).to_string());

    b.iter(|| { board.get_legal_moves(); board.invalidate_caches(); });
}

criterion_group!(benches, perft_benches, bitboard_generation_benches, movegen_bench, piece_benches);
criterion_main!(benches);