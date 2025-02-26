use rand::Rng;
use std::fmt;
use std::io;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum PieceType {
    Flat,
    Standing,
}

#[derive(Clone, Copy, PartialEq)]
enum Player {
    White,
    Black,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Player::White => write!(f, "White"),
            Player::Black => write!(f, "Black"),
        }
    }
}

#[derive(Clone, Copy)]
struct Piece {
    player: Player,
    piece_type: PieceType,
}

struct Stack(Vec<Piece>);

impl Stack {
    fn new() -> Self {
        Stack(Vec::new())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn top_piece(&self) -> Option<&Piece> {
        self.0.last()
    }

    fn controller(&self) -> Option<Player> {
        self.top_piece().map(|p| p.player)
    }

    fn is_blocking(&self) -> bool {
        if let Some(piece) = self.top_piece() {
            piece.piece_type == PieceType::Standing
        } else {
            false
        }
    }

    fn add_piece(&mut self, piece: Piece) {
        self.0.push(piece);
    }

    fn take_n(&mut self, n: usize) -> Vec<Piece> {
        let stack_size = self.0.len();
        if n >= stack_size {
            let result = self.0.clone();
            self.0.clear();
            result
        } else {
            self.0.split_off(stack_size - n)
        }
    }

    fn height(&self) -> usize {
        self.0.len()
    }
}

struct Board {
    size: usize,
    spaces: Vec<Vec<Stack>>,
    white_pieces: usize,
    black_pieces: usize,
    white_caps: usize,
    black_caps: usize,
}

impl Board {
    fn new(size: usize) -> Self {
        let mut spaces = Vec::with_capacity(size);
        for _ in 0..size {
            let mut row = Vec::with_capacity(size);
            for _ in 0..size {
                row.push(Stack::new());
            }
            spaces.push(row);
        }

        // Calculate pieces based on board size
        let (pieces, caps) = match size {
            3 => (10, 0),
            4 => (15, 0),
            5 => (21, 1),
            6 => (30, 1),
            _ => (21, 1), // Default to 5x5 rules
        };

        Board {
            size,
            spaces,
            white_pieces: pieces,
            black_pieces: pieces,
            white_caps: caps,
            black_caps: caps,
        }
    }

    fn place_piece(
        &mut self,
        row: usize,
        col: usize,
        player: Player,
        piece_type: PieceType,
    ) -> Result<(), &'static str> {
        if row >= self.size || col >= self.size {
            return Err("Position out of bounds");
        }

        if !self.spaces[row][col].is_empty() {
            return Err("Space already occupied");
        }

        // Check if player has enough pieces
        match player {
            Player::White => {
                if piece_type == PieceType::Standing && self.white_caps == 0 {
                    return Err("No more capstones available");
                } else if piece_type == PieceType::Flat && self.white_pieces == 0 {
                    return Err("No more flat stones available");
                }
            }
            Player::Black => {
                if piece_type == PieceType::Standing && self.black_caps == 0 {
                    return Err("No more capstones available");
                } else if piece_type == PieceType::Flat && self.black_pieces == 0 {
                    return Err("No more flat stones available");
                }
            }
        }

        // Decrement piece count
        match player {
            Player::White => {
                if piece_type == PieceType::Standing {
                    self.white_caps -= 1;
                } else {
                    self.white_pieces -= 1;
                }
            }
            Player::Black => {
                if piece_type == PieceType::Standing {
                    self.black_caps -= 1;
                } else {
                    self.black_pieces -= 1;
                }
            }
        }

        self.spaces[row][col].add_piece(Piece { player, piece_type });
        Ok(())
    }

    fn move_stack(
        &mut self,
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
        count: usize,
        direction: Direction,
    ) -> Result<(), &'static str> {
        if from_row >= self.size
            || from_col >= self.size
            || to_row >= self.size
            || to_col >= self.size
        {
            return Err("Position out of bounds");
        }

        // Check if the source stack has pieces and who controls it
        if self.spaces[from_row][from_col].is_empty() {
            return Err("No stack to move");
        }

        let controller = self.spaces[from_row][from_col]
            .controller()
            .ok_or("Stack has no controller")?;

        // Validate move distance based on direction
        match direction {
            Direction::North => {
                if from_row <= to_row || from_row - to_row != 1 {
                    return Err("Invalid move distance or direction");
                }
            }
            Direction::South => {
                if from_row >= to_row || to_row - from_row != 1 {
                    return Err("Invalid move distance or direction");
                }
            }
            Direction::East => {
                if from_col >= to_col || to_col - from_col != 1 {
                    return Err("Invalid move distance or direction");
                }
            }
            Direction::West => {
                if from_col <= to_col || from_col - to_col != 1 {
                    return Err("Invalid move distance or direction");
                }
            }
        }

        // Check if target stack is blocked
        if self.spaces[to_row][to_col].is_blocking() {
            return Err("Cannot move onto a blocking piece");
        }

        // Check if player has enough pieces to move
        let source_height = self.spaces[from_row][from_col].height();
        if count > source_height {
            return Err("Not enough pieces in stack to move");
        }

        // Check if player controls this stack
        if self.spaces[from_row][from_col].controller() != Some(controller) {
            return Err("You don't control this stack");
        }

        // Check if player can move that many pieces
        if count > self.size {
            return Err("Cannot move more pieces than board size");
        }

        // Take pieces from source stack
        let pieces_to_move = self.spaces[from_row][from_col].take_n(count);

        // Add pieces to destination stack
        for piece in pieces_to_move {
            self.spaces[to_row][to_col].add_piece(piece);
        }

        Ok(())
    }

    fn check_road_win(&self, player: Player) -> bool {
        // Check horizontal roads
        for row in 0..self.size {
            let mut connected = 0;
            for col in 0..self.size {
                if !self.spaces[row][col].is_empty()
                    && !self.spaces[row][col].is_blocking()
                    && self.spaces[row][col].controller() == Some(player)
                {
                    connected += 1;
                    if connected == self.size {
                        return true;
                    }
                } else {
                    connected = 0;
                }
            }
        }

        // Check vertical roads
        for col in 0..self.size {
            let mut connected = 0;
            for row in 0..self.size {
                if !self.spaces[row][col].is_empty()
                    && !self.spaces[row][col].is_blocking()
                    && self.spaces[row][col].controller() == Some(player)
                {
                    connected += 1;
                    if connected == self.size {
                        return true;
                    }
                } else {
                    connected = 0;
                }
            }
        }

        // Check diagonals (this is a simplified check)
        let mut connected = 0;
        for i in 0..self.size {
            if !self.spaces[i][i].is_empty()
                && !self.spaces[i][i].is_blocking()
                && self.spaces[i][i].controller() == Some(player)
            {
                connected += 1;
                if connected == self.size {
                    return true;
                }
            } else {
                connected = 0;
            }
        }

        connected = 0;
        for i in 0..self.size {
            if !self.spaces[i][self.size - 1 - i].is_empty()
                && !self.spaces[i][self.size - 1 - i].is_blocking()
                && self.spaces[i][self.size - 1 - i].controller() == Some(player)
            {
                connected += 1;
                if connected == self.size {
                    return true;
                }
            } else {
                connected = 0;
            }
        }

        false
    }

    fn is_board_full(&self) -> bool {
        for row in 0..self.size {
            for col in 0..self.size {
                if self.spaces[row][col].is_empty() {
                    return false;
                }
            }
        }
        true
    }

    fn remaining_pieces(&self, player: Player) -> (usize, usize) {
        match player {
            Player::White => (self.white_pieces, self.white_caps),
            Player::Black => (self.black_pieces, self.black_caps),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "  {}",
            (0..self.size)
                .map(|i| format!(" {} ", i))
                .collect::<Vec<_>>()
                .join("")
        )?;

        for row in 0..self.size {
            write!(f, "{} ", row)?;
            for col in 0..self.size {
                let stack = &self.spaces[row][col];
                if stack.is_empty() {
                    write!(f, "[ ]")?;
                } else {
                    let top = stack.top_piece().unwrap();
                    let symbol = match (top.player, top.piece_type) {
                        (Player::White, PieceType::Flat) => "W",
                        (Player::Black, PieceType::Flat) => "B",
                        (Player::White, PieceType::Standing) => "WC",
                        (Player::Black, PieceType::Standing) => "BC",
                    };

                    let height = stack.height();
                    if height > 1 {
                        write!(f, "[{}{}]", symbol, height)?;
                    } else {
                        write!(f, "[{}]", symbol)?;
                    }
                }
            }
            writeln!(f)?;
        }

        // Display remaining pieces
        let (white_pieces, white_caps) = self.remaining_pieces(Player::White);
        let (black_pieces, black_caps) = self.remaining_pieces(Player::Black);

        writeln!(f, "White: {} flat, {} capstone", white_pieces, white_caps)?;
        writeln!(f, "Black: {} flat, {} capstone", black_pieces, black_caps)?;

        Ok(())
    }
}

enum Direction {
    North,
    South,
    East,
    West,
}

enum GameMove {
    Place {
        row: usize,
        col: usize,
        piece_type: PieceType,
    },
    Move {
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
        count: usize,
    },
}

struct Game {
    board: Board,
    current_player: Player,
    turn_number: usize,
}

impl Game {
    fn new(board_size: usize) -> Self {
        Game {
            board: Board::new(board_size),
            current_player: Player::White,
            turn_number: 1,
        }
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

        // Check for win conditions
        if self.board.check_road_win(self.current_player) {
            println!("{} wins by creating a road!", self.current_player);
            return Ok(());
        }

        if self.board.is_board_full() {
            println!("Game over: Board is full!");
            return Ok(());
        }

        // Switch player and increment turn
        self.current_player = match self.current_player {
            Player::White => Player::Black,
            Player::Black => {
                self.turn_number += 1;
                Player::White
            }
        };

        Ok(())
    }

    fn run(&mut self) {
        println!("Welcome to Tak!");
        println!("Board size: {}", self.board.size);

        loop {
            println!(
                "\nTurn {}: {}'s turn",
                self.turn_number, self.current_player
            );
            println!("{}", self.board);

            println!(
                "Enter your move (place row col [flat|cap] or move from_row from_col to_row to_col count):"
            );

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            if parts.len() == 0 {
                println!("Invalid input");
                continue;
            }

            let result = match parts[0] {
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

                    self.make_move(GameMove::Place {
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

                    self.make_move(GameMove::Move {
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
                    println!("Unknown command");
                    continue;
                }
            };

            if let Err(msg) = result {
                println!("Error: {}", msg);
            }

            // Check for win after move
            if self.board.check_road_win(Player::White) {
                println!("{}", self.board);
                println!("White wins by creating a road!");
                break;
            } else if self.board.check_road_win(Player::Black) {
                println!("{}", self.board);
                println!("Black wins by creating a road!");
                break;
            } else if self.board.is_board_full() {
                println!("{}", self.board);
                println!("Game over: Board is full!");
                break;
            }
        }
    }
}

struct AIPlayer {
    player: Player,
    difficulty: usize, // 1-3, with 3 being hardest
}

impl AIPlayer {
    fn new(player: Player, difficulty: usize) -> Self {
        AIPlayer {
            player,
            difficulty: difficulty.clamp(1, 3),
        }
    }

    fn choose_move(&self, game: &Game) -> GameMove {
        let mut rng = rand::thread_rng();

        // Check if it's the opening move or second move
        let opening_phase = game.turn_number <= 2;
        let board_size = game.board.size;

        // 80% chance to make a placement move (except in late game)
        let board_fullness = self.calculate_board_fullness(&game.board);
        let prefer_placement = rng.gen_bool(0.8) && board_fullness < 0.7;

        if opening_phase || prefer_placement {
            // Place a stone
            let piece_type = if self.player == game.current_player
                && rng.gen_bool(0.1)
                && (game.board.remaining_pieces(self.player).1 > 0)
                && !opening_phase
            {
                // 10% chance to place a capstone if available and not in opening
                PieceType::Standing
            } else {
                PieceType::Flat
            };

            // Try to find a good position based on difficulty
            if self.difficulty >= 2 && rng.gen_bool(0.7) {
                // Try to place near the center for better position
                let center = board_size / 2;
                let range = if board_size >= 5 { 2 } else { 1 };

                for _ in 0..5 {
                    // Try 5 times to find a good spot
                    // Ensure we don't go beyond board boundaries
                    let min_row = if center > range { center - range } else { 0 };
                    let max_row = (center + range).min(board_size - 1);
                    let min_col = if center > range { center - range } else { 0 };
                    let max_col = (center + range).min(board_size - 1);

                    let row = rng.gen_range(min_row..=max_row);
                    let col = rng.gen_range(min_col..=max_col);

                    if game.board.spaces[row][col].is_empty() {
                        return GameMove::Place {
                            row,
                            col,
                            piece_type,
                        };
                    }
                }
            }

            // Just find any empty spot
            for _ in 0..20 {
                // Try 20 times to find a random empty spot
                let row = rng.gen_range(0..board_size);
                let col = rng.gen_range(0..board_size);

                if game.board.spaces[row][col].is_empty() {
                    return GameMove::Place {
                        row,
                        col,
                        piece_type,
                    };
                }
            }

            // Fallback: systematically search for an empty space
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

            // If we somehow still haven't found a place, return a random move
            // This shouldn't happen unless board is full
            return GameMove::Place {
                row: rng.gen_range(0..board_size),
                col: rng.gen_range(0..board_size),
                piece_type,
            };
        } else {
            // Move a stack
            // Find a stack we control
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
                // No stacks to move, fall back to placement
                let piece_type = PieceType::Flat;
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
            }

            // Pick a random stack we control
            if let Some(&(from_row, from_col)) = controlled_stacks.choose(&mut rng) {
                let stack_height = game.board.spaces[from_row][from_col].height();
                let count = if stack_height > 1 {
                    rng.gen_range(1..=stack_height.min(board_size))
                } else {
                    1
                };

                // Try to find a valid move
                let possible_moves = [
                    // Check north (row-1)
                    if from_row > 0 {
                        let to_row = from_row - 1;
                        if !game.board.spaces[to_row][from_col].is_blocking() {
                            Some((to_row, from_col, Direction::North))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                    // Check south (row+1)
                    if from_row + 1 < board_size {
                        let to_row = from_row + 1;
                        if !game.board.spaces[to_row][from_col].is_blocking() {
                            Some((to_row, from_col, Direction::South))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                    // Check east (col+1)
                    if from_col + 1 < board_size {
                        let to_col = from_col + 1;
                        if !game.board.spaces[from_row][to_col].is_blocking() {
                            Some((from_row, to_col, Direction::East))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                    // Check west (col-1)
                    if from_col > 0 {
                        let to_col = from_col - 1;
                        if !game.board.spaces[from_row][to_col].is_blocking() {
                            Some((from_row, to_col, Direction::West))
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                ];

                let mut valid_moves = Vec::new();

                for option in &possible_moves {
                    if let Some((to_row, to_col, _)) = option {
                        valid_moves.push((*to_row, *to_col));
                    }
                }

                if let Some(&(to_row, to_col)) = valid_moves.choose(&mut rng) {
                    return GameMove::Move {
                        from_row,
                        from_col,
                        to_row,
                        to_col,
                        count,
                    };
                }
            }

            // If we can't move, place a stone
            let piece_type = PieceType::Flat;
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

            // This should never happen unless the board is full
            return GameMove::Place {
                row: 0,
                col: 0,
                piece_type: PieceType::Flat,
            };
        }
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
}

// Add a choose method for Vec
trait Choose<T> {
    fn choose(&self, rng: &mut rand::rngs::ThreadRng) -> Option<&T>;
}

impl<T> Choose<T> for Vec<T> {
    fn choose(&self, rng: &mut rand::rngs::ThreadRng) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self[rng.gen_range(0..self.len())])
        }
    }
}

impl Game {
    // Add a method to run the game with AI players
    fn run_ai_vs_ai(&mut self, white_difficulty: usize, black_difficulty: usize, delay_ms: u64) {
        println!("Welcome to Tak AI vs AI!");
        println!("Board size: {}", self.board.size);
        println!("White AI difficulty: {}", white_difficulty);
        println!("Black AI difficulty: {}", black_difficulty);

        let white_ai = AIPlayer::new(Player::White, white_difficulty);
        let black_ai = AIPlayer::new(Player::Black, black_difficulty);

        loop {
            println!(
                "\nTurn {}: {}'s turn",
                self.turn_number, self.current_player
            );
            println!("{}", self.board);

            // Add delay for better visualization
            thread::sleep(Duration::from_millis(delay_ms));

            let ai_move = if self.current_player == Player::White {
                white_ai.choose_move(self)
            } else {
                black_ai.choose_move(self)
            };

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
                    println!(
                        "{} AI places a {} at row {}, column {}",
                        self.current_player, piece_name, row, col
                    );
                }
                GameMove::Move {
                    from_row,
                    from_col,
                    to_row,
                    to_col,
                    count,
                } => {
                    println!(
                        "{} AI moves {} pieces from ({},{}) to ({},{})",
                        self.current_player, count, from_row, from_col, to_row, to_col
                    );
                }
            }

            let result = self.make_move(ai_move);

            if let Err(msg) = result {
                println!("Error in AI move: {}", msg);
                // If AI made an invalid move, let it try again with a placement
                let fallback_move = GameMove::Place {
                    row: 0,
                    col: 0,
                    piece_type: PieceType::Flat,
                };
                if let Err(fallback_msg) = self.make_move(fallback_move) {
                    println!("AI fallback move also failed: {}", fallback_msg);
                    // If even the fallback fails, just end the game
                    break;
                }
            }

            // Check for win after move
            if self.board.check_road_win(Player::White) {
                println!("{}", self.board);
                println!("White wins by creating a road!");
                break;
            } else if self.board.check_road_win(Player::Black) {
                println!("{}", self.board);
                println!("Black wins by creating a road!");
                break;
            } else if self.board.is_board_full() {
                println!("{}", self.board);
                println!("Game over: Board is full!");
                break;
            }

            // Check for excessive turns to prevent infinite games
            if self.turn_number > 100 {
                println!("Game terminated after 100 turns");
                break;
            }
        }
    }
}

fn main() {
    println!("Welcome to Tak Simulator!");
    println!("Game modes:");
    println!("1. Human vs Human");
    println!("2. AI vs AI");
    println!("Choose a mode (1 or 2):");

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let mode = input.trim().parse::<usize>().unwrap_or(1);

    println!("Enter board size (3-6):");
    input.clear();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let size = input.trim().parse::<usize>().unwrap_or(5);
    let board_size = if size < 3 || size > 6 {
        println!("Invalid board size, using default 5x5");
        5
    } else {
        size
    };

    let mut game = Game::new(board_size);

    match mode {
        1 => game.run(),
        2 => {
            println!("Enter White AI difficulty (1-3):");
            input.clear();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let white_difficulty = input.trim().parse::<usize>().unwrap_or(2);

            println!("Enter Black AI difficulty (1-3):");
            input.clear();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let black_difficulty = input.trim().parse::<usize>().unwrap_or(2);

            println!("Enter delay between moves in milliseconds (recommend 500-2000):");
            input.clear();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let delay = input.trim().parse::<u64>().unwrap_or(1000);

            game.run_ai_vs_ai(white_difficulty, black_difficulty, delay);
        }
        _ => {
            println!("Invalid mode, using Human vs Human");
            game.run();
        }
    }
}
