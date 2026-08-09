use crate::board::Board;

impl Board {
    pub fn test_duplicate_moves(&mut self) {
        let legal_moves = self.get_legal_moves().as_ref().expect("Unable to get legal moves");

        for i in 0..legal_moves.len() {
            for j in 0..legal_moves.len() {
                if i != j {
                    if legal_moves[i] == legal_moves[j] {
                        eprintln!("Move duplicate: {} -> {}",Self::square_index_to_chars(legal_moves[i].origin_square).iter().collect::<String>(),
                            Self::square_index_to_chars(legal_moves[j].destination_square).iter().collect::<String>());
                    }
                }
            }
        }
    }

    pub fn test_fen_string_handling(&mut self) {
        let test_fen_strings: [String; 6] = ["rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()/*, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -".to_string()*/, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string(), "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(), "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string(), "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".to_string(), "r1bqkb1r/pppp1pp1/2n2n1p/4p3/2B1PP2/2NP4/PPP3PP/R1BQK1NR b KQkq f3 0 5".to_string()];

        let mut failure_count = 0;

        for test_fen in test_fen_strings.iter() {
            let mut board = Board::new(None);

            board.load_fen(test_fen);

            let result = board.generate_fen();

            if !result.eq(test_fen) {
                failure_count += 1;
                eprintln!("Input : {}", test_fen);
                eprintln!("Result: {}", result);
            }
        }

        if failure_count > 0 {
            println!("{} Test failed", failure_count);
        }
        else {
            println!("All fen tests succeeded");
        }
    }
}