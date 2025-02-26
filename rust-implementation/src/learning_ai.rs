use crate::{Board, Direction, Game, GameMove, PieceType, Player, Stack};
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

// Self-learning AI implementation
pub struct SelfLearningAI {
    pub player: Player,
    pub learning_rate: f64,
    // Strategy weights that will be adjusted during training
    pub strategy_weights: HashMap<String, f64>,
    // Training statistics
    pub games_played: usize,
    pub games_won: usize,
}

impl SelfLearningAI {
    pub fn new(player: Player, learning_rate: f64) -> Self {
        let mut weights = HashMap::new();

        // Initialize weights with default values
        weights.insert("place_center".to_string(), 0.8);
        weights.insert("place_edge".to_string(), 0.5);
        weights.insert("place_corner".to_string(), 0.3);
        weights.insert("place_middle".to_string(), 0.5);
        weights.insert("use_capstone".to_string(), 0.4);
        weights.insert("stack_move_small".to_string(), 0.6);
        weights.insert("stack_move_large".to_string(), 0.7);
        weights.insert("move_north".to_string(), 0.5);
        weights.insert("move_south".to_string(), 0.5);
        weights.insert("move_east".to_string(), 0.5);
        weights.insert("move_west".to_string(), 0.5);
        weights.insert("place_action".to_string(), 0.7);
        weights.insert("move_action".to_string(), 0.3);

        SelfLearningAI {
            player,
            learning_rate,
            strategy_weights: weights,
            games_played: 0,
            games_won: 0,
        }
    }

    // Choose a move based on learned weights
    pub fn choose_move(&self, game: &Game) -> GameMove {
        let mut rng = rand::thread_rng();
        let board_size = game.board.size;

        // Decide whether to place or move
        let board_fullness = self.calculate_board_fullness(&game.board);

        // Early game or empty board: always place
        if game.turn_number <= 2 || board_fullness < 0.3 {
            return self.choose_placement(game);
        }

        // Otherwise, weighted choice between place and move
        let place_weight = self.get_weight("place_action");
        let move_weight = self.get_weight("move_action");
        let total_weight = place_weight + move_weight;

        let place_probability = place_weight / total_weight;
        let should_place = rng.gen_bool(place_probability);

        if should_place {
            self.choose_placement(game)
        } else {
            match self.choose_stack_move(game) {
                Some(move_action) => move_action,
                None => self.choose_placement(game), // Fallback to placement
            }
        }
    }

    fn choose_placement(&self, game: &Game) -> GameMove {
        let mut rng = rand::thread_rng();
        let board_size = game.board.size;

        // Decide on piece type (flat stone or capstone)
        let use_capstone_weight = self.get_weight("use_capstone");
        let use_capstone = rng.gen_bool(use_capstone_weight)
            && game.board.remaining_pieces(self.player).1 > 0
            && game.turn_number > 2; // No capstones in opening

        let piece_type = if use_capstone {
            PieceType::Standing
        } else {
            PieceType::Flat
        };

        // Choose placement location based on weights
        let center_weight = self.get_weight("place_center");
        let edge_weight = self.get_weight("place_edge");
        let corner_weight = self.get_weight("place_corner");
        let middle_weight = self.get_weight("place_middle");

        let total_weight = center_weight + edge_weight + corner_weight + middle_weight;

        let center_prob = center_weight / total_weight;
        let edge_prob = edge_weight / total_weight;
        let corner_prob = corner_weight / total_weight;

        // Determine placement zone
        let placement_zone = {
            let roll = rng.gen_range(0.0..1.0);
            if roll < center_prob {
                "center"
            } else if roll < center_prob + edge_prob {
                "edge"
            } else if roll < center_prob + edge_prob + corner_prob {
                "corner"
            } else {
                "middle"
            }
        };

        // Find available positions in the chosen zone
        let center = board_size / 2;
        let mut available_positions = Vec::new();

        match placement_zone {
            "center" => {
                let c_range = if board_size >= 4 { 1 } else { 0 };
                for r in (center.saturating_sub(c_range))..=(center + c_range).min(board_size - 1) {
                    for c in
                        (center.saturating_sub(c_range))..=(center + c_range).min(board_size - 1)
                    {
                        if game.board.spaces[r][c].is_empty() {
                            available_positions.push((r, c));
                        }
                    }
                }
            }
            "edge" => {
                for r in 0..board_size {
                    for c in 0..board_size {
                        if (r == 0 || r == board_size - 1 || c == 0 || c == board_size - 1)
                            && !(r == 0 && c == 0)
                            && !(r == 0 && c == board_size - 1)
                            && !(r == board_size - 1 && c == 0)
                            && !(r == board_size - 1 && c == board_size - 1)
                        {
                            if game.board.spaces[r][c].is_empty() {
                                available_positions.push((r, c));
                            }
                        }
                    }
                }
            }
            "corner" => {
                let corners = [
                    (0, 0),
                    (0, board_size - 1),
                    (board_size - 1, 0),
                    (board_size - 1, board_size - 1),
                ];
                for &(r, c) in &corners {
                    if game.board.spaces[r][c].is_empty() {
                        available_positions.push((r, c));
                    }
                }
            }
            "middle" => {
                for r in 1..(board_size - 1) {
                    for c in 1..(board_size - 1) {
                        if game.board.spaces[r][c].is_empty()
                            && !((r == center || r == center - 1 || r == center + 1)
                                && (c == center || c == center - 1 || c == center + 1))
                        {
                            available_positions.push((r, c));
                        }
                    }
                }
            }
            _ => {}
        }

        // If no positions available in chosen zone, try all empty spaces
        if available_positions.is_empty() {
            for r in 0..board_size {
                for c in 0..board_size {
                    if game.board.spaces[r][c].is_empty() {
                        available_positions.push((r, c));
                    }
                }
            }
        }

        // Select a random position from available positions
        if !available_positions.is_empty() {
            let idx = rng.gen_range(0..available_positions.len());
            let (row, col) = available_positions[idx];
            return GameMove::Place {
                row,
                col,
                piece_type,
            };
        }

        // Fallback to a random position if somehow we didn't find any
        for row in 0..board_size {
            for col in 0..board_size {
                if game.board.spaces[row][col].is_empty() {
                    return GameMove::Place {
                        row,
                        col,
                        piece_type,
                    };
                }
            }
        }

        // Ultimate fallback
        GameMove::Place {
            row: 0,
            col: 0,
            piece_type: PieceType::Flat,
        }
    }

    fn choose_stack_move(&self, game: &Game) -> Option<GameMove> {
        let mut rng = rand::thread_rng();
        let board_size = game.board.size;

        // Find all stacks that this player controls
        let mut controlled_stacks = Vec::new();
        for row in 0..board_size {
            for col in 0..board_size {
                if !game.board.spaces[row][col].is_empty()
                    && game.board.spaces[row][col].controller() == Some(self.player)
                {
                    controlled_stacks.push((row, col));
                }
            }
        }

        if controlled_stacks.is_empty() {
            return None;
        }

        // Get directional weights
        let north_weight = self.get_weight("move_north");
        let south_weight = self.get_weight("move_south");
        let east_weight = self.get_weight("move_east");
        let west_weight = self.get_weight("move_west");

        // Get stack size preference weights
        let small_stack_weight = self.get_weight("stack_move_small");
        let large_stack_weight = self.get_weight("stack_move_large");

        // Choose a random stack based on controlled stacks
        if !controlled_stacks.is_empty() {
            let idx = rng.gen_range(0..controlled_stacks.len());
            let (from_row, from_col) = controlled_stacks[idx];

            let stack_height = game.board.spaces[from_row][from_col].height();

            // Decide how many pieces to move
            let max_move = stack_height.min(board_size);
            let prefer_small =
                rng.gen_bool(small_stack_weight / (small_stack_weight + large_stack_weight));

            let count = if prefer_small || max_move <= 1 {
                1 // Move just the top piece
            } else if max_move == 2 {
                2 // If only 2 pieces, move both
            } else {
                // For larger stacks, use weighted randomness
                let small_range = 1..=(max_move / 2).max(1);
                let large_range = (max_move / 2 + 1)..=max_move;

                if prefer_small && *small_range.end() >= *small_range.start() {
                    rng.gen_range(small_range)
                } else if *large_range.end() >= *large_range.start() {
                    rng.gen_range(large_range)
                } else {
                    1 // Fallback
                }
            };

            // Determine possible directions to move
            let mut possible_directions = Vec::new();

            // Check north (row-1)
            if from_row > 0 && !game.board.spaces[from_row - 1][from_col].is_blocking() {
                possible_directions.push(("north", from_row - 1, from_col, north_weight));
            }

            // Check south (row+1)
            if from_row + 1 < board_size && !game.board.spaces[from_row + 1][from_col].is_blocking()
            {
                possible_directions.push(("south", from_row + 1, from_col, south_weight));
            }

            // Check east (col+1)
            if from_col + 1 < board_size && !game.board.spaces[from_row][from_col + 1].is_blocking()
            {
                possible_directions.push(("east", from_row, from_col + 1, east_weight));
            }

            // Check west (col-1)
            if from_col > 0 && !game.board.spaces[from_row][from_col - 1].is_blocking() {
                possible_directions.push(("west", from_row, from_col - 1, west_weight));
            }

            if possible_directions.is_empty() {
                return None; // No valid moves from this stack
            }

            // Select a direction using weighted probability
            let total_weight: f64 = possible_directions.iter().map(|(_, _, _, w)| w).sum();
            let mut cumulative_weight = 0.0;
            let roll = rng.gen_range(0.0..total_weight);

            for (_, to_row, to_col, weight) in &possible_directions {
                cumulative_weight += weight;
                if roll <= cumulative_weight {
                    return Some(GameMove::Move {
                        from_row,
                        from_col,
                        to_row: *to_row,
                        to_col: *to_col,
                        count,
                    });
                }
            }

            // Fallback in case weight calculation had issues
            if !possible_directions.is_empty() {
                let idx = rng.gen_range(0..possible_directions.len());
                let (_, to_row, to_col, _) = possible_directions[idx];
                return Some(GameMove::Move {
                    from_row,
                    from_col,
                    to_row,
                    to_col,
                    count,
                });
            }
        }

        None
    }

    fn get_weight(&self, strategy: &str) -> f64 {
        *self.strategy_weights.get(strategy).unwrap_or(&0.5)
    }

    fn calculate_board_fullness(&self, board: &Board) -> f64 {
        let mut occupied = 0;
        let total = board.size * board.size;

        for row in 0..board.size {
            for col in 0..board.size {
                if !board.spaces[row][col].is_empty() {
                    occupied += 1;
                }
            }
        }

        occupied as f64 / total as f64
    }

    // Update weights based on game outcome
    pub fn update_weights(&mut self, won: bool, moves_history: &[(String, GameMove)]) {
        self.games_played += 1;
        if won {
            self.games_won += 1;
        }

        // Calculate reward
        let reward = if won { 1.0 } else { -0.5 };

        // Update weights for each move in history
        for (strategy, _) in moves_history {
            if let Some(weight) = self.strategy_weights.get_mut(strategy) {
                // Update weight: move toward reward based on learning rate
                *weight += self.learning_rate * (reward - *weight);

                // Ensure weight stays within reasonable bounds
                *weight = weight.max(0.1).min(1.0);
            }
        }
    }

    // Load weights from a file
    pub fn load_weights(&mut self, filename: &str) -> Result<(), std::io::Error> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    self.strategy_weights.insert(key, value);
                }
            }
        }

        Ok(())
    }

    // Save weights to a file
    pub fn save_weights(&self, filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::create(filename)?;

        for (key, value) in &self.strategy_weights {
            writeln!(file, "{}:{}", key, value)?;
        }

        Ok(())
    }

    // Print training statistics
    pub fn print_stats(&self) {
        println!("Training stats for {:?} AI:", self.player);
        println!("Games played: {}", self.games_played);
        println!(
            "Games won: {} ({}%)",
            self.games_won,
            if self.games_played > 0 {
                (self.games_won as f64 / self.games_played as f64 * 100.0).round()
            } else {
                0.0
            }
        );
        println!("Current strategy weights:");

        let mut weights: Vec<(&String, &f64)> = self.strategy_weights.iter().collect();
        weights.sort_by(|a, b| a.0.cmp(b.0));

        for (strategy, weight) in weights {
            println!("  {}: {:.2}", strategy, weight);
        }
    }
}

// Make SelfLearningAI cloneable
impl Clone for SelfLearningAI {
    fn clone(&self) -> Self {
        SelfLearningAI {
            player: self.player,
            learning_rate: self.learning_rate,
            strategy_weights: self.strategy_weights.clone(),
            games_played: self.games_played,
            games_won: self.games_won,
        }
    }
}

// Add this struct to track moves during training
pub struct TrainingGame {
    pub board: Board,
    pub current_player: Player,
    pub turn_number: usize,
    pub white_ai: SelfLearningAI,
    pub black_ai: SelfLearningAI,
    pub white_moves: Vec<(String, GameMove)>,
    pub black_moves: Vec<(String, GameMove)>,
    pub max_turns: usize,
}

impl TrainingGame {
    pub fn new(board_size: usize, white_lr: f64, black_lr: f64, max_turns: usize) -> Self {
        TrainingGame {
            board: Board::new(board_size),
            current_player: Player::White,
            turn_number: 1,
            white_ai: SelfLearningAI::new(Player::White, white_lr),
            black_ai: SelfLearningAI::new(Player::Black, black_lr),
            white_moves: Vec::new(),
            black_moves: Vec::new(),
            max_turns,
        }
    }

    pub fn run_training_game(&mut self, verbose: bool) -> Option<Player> {
        if verbose {
            println!("Starting training game");
            println!("Board size: {}", self.board.size);
        }

        let mut winner = None;

        loop {
            if verbose {
                println!(
                    "\nTurn {}: {}'s turn",
                    self.turn_number, self.current_player
                );
                println!("{}", self.board);
            }

            // Choose a move based on the current player
            let (ai_move, strategy) = if self.current_player == Player::White {
                self.choose_move_with_strategy(&self.white_ai)
            } else {
                self.choose_move_with_strategy(&self.black_ai)
            };

            // Record the move in history
            if self.current_player == Player::White {
                self.white_moves.push((strategy, ai_move.clone()));
            } else {
                self.black_moves.push((strategy, ai_move.clone()));
            }

            // Execute the move
            let result = self.make_move(ai_move);

            if let Err(msg) = result {
                if verbose {
                    println!("Error in AI move: {}", msg);
                }

                // If AI made an invalid move, try a simple placement
                let mut fallback_successful = false;
                'outer: for row in 0..self.board.size {
                    for col in 0..self.board.size {
                        if self.board.spaces[row][col].is_empty() {
                            let fallback_move = GameMove::Place {
                                row,
                                col,
                                piece_type: PieceType::Flat,
                            };

                            if verbose {
                                println!("AI attempts fallback move: place {} {} flat", row, col);
                            }

                            if self.make_move(fallback_move).is_ok() {
                                fallback_successful = true;
                                break 'outer;
                            }
                        }
                    }
                }

                // If no valid move could be found, we should handle this case
                if !fallback_successful {
                    if verbose {
                        println!("No valid moves available - ending game");
                    }
                    break;
                }
            }

            // Check for win conditions
            if self.board.check_road_win(Player::White) {
                if verbose {
                    println!("{}", self.board);
                    println!("White wins by creating a road!");
                }
                winner = Some(Player::White);
                break;
            } else if self.board.check_road_win(Player::Black) {
                if verbose {
                    println!("{}", self.board);
                    println!("Black wins by creating a road!");
                }
                winner = Some(Player::Black);
                break;
            } else if self.board.is_board_full() {
                if verbose {
                    println!("{}", self.board);
                    println!("Game over: Board is full!");
                }
                // In case of a draw, no winner
                break;
            }

            // Check for excessive turns
            if self.turn_number > self.max_turns {
                if verbose {
                    println!("Game terminated after {} turns", self.max_turns);
                }
                break;
            }

            // Switch player and increment turn
            self.current_player = match self.current_player {
                Player::White => Player::Black,
                Player::Black => {
                    self.turn_number += 1;
                    Player::White
                }
            };
        }

        // Update AI weights based on game outcome
        if let Some(winner) = winner {
            self.white_ai
                .update_weights(winner == Player::White, &self.white_moves);
            self.black_ai
                .update_weights(winner == Player::Black, &self.black_moves);
        }

        winner
    }

    fn choose_move_with_strategy(&self, ai: &SelfLearningAI) -> (GameMove, String) {
        // Create a Game object from the current TrainingGame state
        let game = Game {
            board: self.board.clone(),
            current_player: self.current_player,
            turn_number: self.turn_number,
        };

        // Pass the Game reference to the AI
        let game_move = ai.choose_move(&game);

        // Determine which strategy was used
        let strategy = match &game_move {
            GameMove::Place {
                row,
                col,
                piece_type,
            } => {
                let center = self.board.size / 2;
                let is_center = (*row as i32 - center as i32).abs() <= 1
                    && (*col as i32 - center as i32).abs() <= 1;

                let is_edge = *row == 0
                    || *row == self.board.size - 1
                    || *col == 0
                    || *col == self.board.size - 1;

                let is_corner = (*row == 0 || *row == self.board.size - 1)
                    && (*col == 0 || *col == self.board.size - 1);

                match piece_type {
                    PieceType::Standing => "use_capstone".to_string(),
                    PieceType::Flat => {
                        if is_corner {
                            "place_corner".to_string()
                        } else if is_edge {
                            "place_edge".to_string()
                        } else if is_center {
                            "place_center".to_string()
                        } else {
                            "place_middle".to_string()
                        }
                    }
                }
            }
            GameMove::Move {
                from_row,
                from_col,
                to_row,
                to_col,
                count,
            } => {
                let direction = if to_row < from_row {
                    "move_north"
                } else if to_row > from_row {
                    "move_south"
                } else if to_col > from_col {
                    "move_east"
                } else {
                    "move_west"
                };

                if *count <= self.board.size / 2 {
                    format!("{}_{}", "stack_move_small", direction)
                } else {
                    format!("{}_{}", "stack_move_large", direction)
                }
            }
        };

        (game_move, strategy)
    }

    fn make_move(&mut self, game_move: GameMove) -> Result<(), &'static str> {
        match game_move {
            GameMove::Place {
                row,
                col,
                piece_type,
            } => {
                self.board
                    .place_piece(row, col, self.current_player, piece_type)?;
            }
            GameMove::Move {
                from_row,
                from_col,
                to_row,
                to_col,
                count,
            } => {
                // Determine direction
                let direction = if to_row < from_row {
                    Direction::North
                } else if to_row > from_row {
                    Direction::South
                } else if to_col > from_col {
                    Direction::East
                } else {
                    Direction::West
                };

                self.board
                    .move_stack(from_row, from_col, to_row, to_col, count, direction)?;
            }
        }

        Ok(())
    }

    // Get the trained AI instances
    pub fn get_trained_ais(&self) -> (SelfLearningAI, SelfLearningAI) {
        (self.white_ai.clone(), self.black_ai.clone())
    }
}

// Public functions for training and playing
pub fn train_ai(
    board_size: usize,
    num_games: usize,
    white_lr: f64,
    black_lr: f64,
    max_turns: usize,
    verbose: bool,
) -> (SelfLearningAI, SelfLearningAI) {
    let mut white_wins = 0;
    let mut black_wins = 0;
    let mut draws = 0;

    let mut game = TrainingGame::new(board_size, white_lr, black_lr, max_turns);

    println!("Starting training session for {} games...", num_games);

    for i in 1..=num_games {
        // Create a new game board but keep the AI learning
        game.board = Board::new(board_size);
        game.current_player = Player::White;
        game.turn_number = 1;
        game.white_moves.clear();
        game.black_moves.clear();

        let winner = game.run_training_game(verbose && i % 100 == 0);

        match winner {
            Some(Player::White) => white_wins += 1,
            Some(Player::Black) => black_wins += 1,
            None => draws += 1,
        }

        if i % 100 == 0 || i == num_games {
            println!("Training progress: {}/{} games", i, num_games);
            println!(
                "  White wins: {} ({}%)",
                white_wins,
                (white_wins as f64 / i as f64 * 100.0).round()
            );
            println!(
                "  Black wins: {} ({}%)",
                black_wins,
                (black_wins as f64 / i as f64 * 100.0).round()
            );
            println!(
                "  Draws: {} ({}%)",
                draws,
                (draws as f64 / i as f64 * 100.0).round()
            );
        }
    }

    println!("\nTraining complete!");
    println!("Final results after {} games:", num_games);
    println!(
        "  White wins: {} ({}%)",
        white_wins,
        (white_wins as f64 / num_games as f64 * 100.0).round()
    );
    println!(
        "  Black wins: {} ({}%)",
        black_wins,
        (black_wins as f64 / num_games as f64 * 100.0).round()
    );
    println!(
        "  Draws: {} ({}%)",
        draws,
        (draws as f64 / num_games as f64 * 100.0).round()
    );

    // Print the final weights
    println!("\nWhite AI final weights:");
    game.white_ai.print_stats();

    println!("\nBlack AI final weights:");
    game.black_ai.print_stats();

    // Return the trained AIs
    game.get_trained_ais()
}

pub fn run_human_vs_trained_ai(
    board_size: usize,
    white_ai: SelfLearningAI,
    black_ai: SelfLearningAI,
) {
    println!("Do you want to play as White or Black? (white/black):");
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let human_player = match input.trim().to_lowercase().as_str() {
        "black" => Player::Black,
        _ => Player::White, // Default to White
    };

    let ai = match human_player {
        Player::White => black_ai,
        Player::Black => white_ai,
    };

    run_human_vs_learning_ai(board_size, human_player, ai);
}

pub fn run_human_vs_learning_ai(board_size: usize, human_player: Player, ai: SelfLearningAI) {
    println!("Welcome to Tak - Human vs Trained AI!");
    println!("Board size: {}", board_size);
    println!("You are playing as {}", human_player);

    let mut game = Game::new(board_size);
    let ai_player = match human_player {
        Player::White => Player::Black,
        Player::Black => Player::White,
    };

    // First, display the AI's learned weights
    println!("\nTrained AI strategy weights:");
    ai.print_stats();

    // Game loop
    loop {
        println!(
            "\nTurn {}: {}'s turn",
            game.turn_number, game.current_player
        );
        println!("{}", game.board);

        let result = if game.current_player == ai_player {
            // AI's turn
            println!("AI is thinking...");
            std::thread::sleep(std::time::Duration::from_millis(1000)); // Simulate thinking

            let ai_move = ai.choose_move(&game);

            // Display the AI's move
            match &ai_move {
                GameMove::Place {
                    row,
                    col,
                    piece_type,
                } => {
                    let piece_name = match piece_type {
                        PieceType::Flat => "flat stone",
                        PieceType::Standing => "capstone",
                    };
                    println!("AI places a {} at row {}, column {}", piece_name, row, col);
                }
                GameMove::Move {
                    from_row,
                    from_col,
                    to_row,
                    to_col,
                    count,
                } => {
                    println!(
                        "AI moves {} pieces from ({},{}) to ({},{})",
                        count, from_row, from_col, to_row, to_col
                    );
                }
            }

            game.make_move(ai_move)
        } else {
            // Human's turn
            println!(
                "Enter your move (place row col [flat|cap] or move from_row from_col to_row to_col count):"
            );

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            if parts.is_empty() {
                println!("Invalid input");
                continue;
            }

            match parts[0] {
                "place" => {
                    if parts.len() != 4 {
                        println!("Invalid place command format");
                        continue;
                    }

                    let row = parts[1].parse::<usize>().unwrap_or(999);
                    let col = parts[2].parse::<usize>().unwrap_or(999);
                    let piece_type = match parts[3] {
                        "flat" => PieceType::Flat,
                        "cap" => PieceType::Standing,
                        _ => {
                            println!("Invalid piece type");
                            continue;
                        }
                    };

                    game.make_move(GameMove::Place {
                        row,
                        col,
                        piece_type,
                    })
                }
                "move" => {
                    if parts.len() != 6 {
                        println!("Invalid move command format");
                        continue;
                    }

                    let from_row = parts[1].parse::<usize>().unwrap_or(999);
                    let from_col = parts[2].parse::<usize>().unwrap_or(999);
                    let to_row = parts[3].parse::<usize>().unwrap_or(999);
                    let to_col = parts[4].parse::<usize>().unwrap_or(999);
                    let count = parts[5].parse::<usize>().unwrap_or(0);

                    game.make_move(GameMove::Move {
                        from_row,
                        from_col,
                        to_row,
                        to_col,
                        count,
                    })
                }
                "quit" => {
                    println!("Thanks for playing!");
                    break;
                }
                _ => {
                    println!("Unknown command. Valid commands are 'place', 'move', or 'quit'.");
                    continue;
                }
            }
        };

        if let Err(msg) = result {
            println!("Error: {}", msg);
            continue;
        }

        // Check for win after move
        if game.board.check_road_win(Player::White) {
            println!("{}", game.board);
            println!("White wins by creating a road!");
            break;
        } else if game.board.check_road_win(Player::Black) {
            println!("{}", game.board);
            println!("Black wins by creating a road!");
            break;
        } else if game.board.is_board_full() {
            println!("{}", game.board);
            println!("Game over: Board is full!");
            break;
        }

        // Switch player
        game.current_player = match game.current_player {
            Player::White => Player::Black,
            Player::Black => {
                game.turn_number += 1;
                Player::White
            }
        };
    }
}
