//! 結合テスト: CPU同士の対戦が終局まで進むことを確認する。

/// 統合テスト本体。
#[cfg(test)]
mod tests {
    use gemini_core::ai::types::Ai;
    use gemini_core::{ai, engine};

    /// `alphabeta` が合法手のみ選ぶことを確認する。
    #[test]
    fn alphabeta_selects_legal_move() {
        let position = engine::Position::initial();
        let legal_moves = position.legal_moves();
        assert!(
            legal_moves != u64::MIN,
            "initial position must have legal moves"
        );

        let mut agent = ai::alphabeta::Agent::new(3);
        let mv = agent.select_move(position);
        assert!(
            matches!(mv, ai::Move::Place(_)),
            "alphabeta must not pass in initial position, got={mv:?}"
        );
        let square = match mv {
            ai::Move::Place(value) => value,
            ai::Move::Pass => return,
            _ => return,
        };

        assert!(
            legal_moves & square.bit() != u64::MIN,
            "alphabeta must select a legal move, got={square:?}"
        );
    }

    /// `random vs alphabeta` で終局することを確認する。
    fn play_game_random_vs_alphabeta(seed_black: u64, depth_white: u8, seed_white: u64) {
        let mut game = engine::Game::initial();
        let mut black_agent = ai::random::Agent::new(seed_black);
        let mut white_agent = ai::alphabeta::Agent::new(depth_white);
        let _: u64 = seed_white;

        // リバーシは最大60手（最初の4石を除く）だが、パスもあるので余裕を見て回す。
        for _turn in u16::MIN..200 {
            let position = game.position();

            let mv = match game.side_to_move() {
                engine::Color::Black => black_agent.select_move(position),
                engine::Color::White => white_agent.select_move(position),
                _ => ai::Move::Pass,
            };

            let play_result = match mv {
                ai::Move::Pass => game.play(None),
                ai::Move::Place(square) => game.play(Some(square)),
                _ => game.play(None),
            };

            assert!(
                play_result.is_ok(),
                "play must succeed, got={play_result:?}"
            );

            let status = match play_result {
                Ok(value) => value,
                Err(_err) => return,
            };

            if let engine::GameStatus::GameOver {
                black: black_count,
                white: white_count,
            } = status
            {
                let total_opt = black_count.checked_add(white_count);
                assert!(total_opt.is_some(), "black+white must not overflow");

                let total = match total_opt {
                    Some(value) => value,
                    None => return,
                };

                assert!(total <= 64, "total stones must be <= 64, got={total}");
                return;
            }
        }

        let status = game.status();
        assert!(
            matches!(status, engine::GameStatus::GameOver { .. }),
            "game did not finish within turn limit, status={status:?}"
        );
    }

    /// `random` 同士で終局することを確認する。
    fn play_game_random_vs_random(seed_black: u64, seed_white: u64) {
        let mut game = engine::Game::initial();
        let mut black_agent = ai::random::Agent::new(seed_black);
        let mut white_agent = ai::random::Agent::new(seed_white);

        // リバーシは最大60手（最初の4石を除く）だが、パスもあるので余裕を見て回す。
        for _turn in u16::MIN..200 {
            let position = game.position();

            let mv = match game.side_to_move() {
                engine::Color::Black => black_agent.select_move(position),
                engine::Color::White => white_agent.select_move(position),
                _ => ai::Move::Pass,
            };

            let play_result = match mv {
                ai::Move::Pass => game.play(None),
                ai::Move::Place(square) => game.play(Some(square)),
                _ => game.play(None),
            };

            assert!(
                play_result.is_ok(),
                "play must succeed, got={play_result:?}"
            );

            let status = match play_result {
                Ok(value) => value,
                Err(_err) => return,
            };

            if let engine::GameStatus::GameOver {
                black: black_count,
                white: white_count,
            } = status
            {
                // 石数は最大64で、重複は起きないはず。
                let total_opt = black_count.checked_add(white_count);
                assert!(total_opt.is_some(), "black+white must not overflow");

                let total = match total_opt {
                    Some(value) => value,
                    None => return,
                };

                assert!(total <= 64, "total stones must be <= 64, got={total}");
                return;
            }
        }

        let status = game.status();
        assert!(
            matches!(status, engine::GameStatus::GameOver { .. }),
            "game did not finish within turn limit, status={status:?}"
        );
    }

    /// `random vs alphabeta` が終局まで進む。
    #[test]
    fn random_vs_alphabeta_finishes() {
        play_game_random_vs_alphabeta(u64::MIN, u8::MIN.wrapping_add(1), u64::MIN);
        play_game_random_vs_alphabeta(42, 3, 0);
    }

    /// `random vs random` が終局まで進む。
    #[test]
    fn random_vs_random_finishes() {
        play_game_random_vs_random(u64::MIN, u64::MIN.wrapping_add(1));
        play_game_random_vs_random(42, 4242);
    }
}
