// Game structs and types - put these at the top of your main.rs file
use rand::Rng;
use std::fmt;
use std::io;
use std::thread;
use std::time::Duration;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Player {
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceType {
    Flat,
    Standing,
}

#[derive(Debug, Clone)]
pub struct Piece {
    player: Player,
    piece_type: PieceType,
}

impl Piece {
    pub fn new(player: Player, piece_type: PieceType) -> Self {
        Piece { player, piece_type }
    }

    pub fn is_flat(&self) -> bool {
        self.piece_type == PieceType::Flat
    }

    pub fn is_standing(&self) -> bool {
        self.piece_type == PieceType::Standing
    }
}

#[derive(Debug, Clone)]
pub struct Stack {
    pieces: Vec<Piece>,
}

impl Stack {
    pub fn new() -> Self {
        Stack { pieces: Vec::new() }
    }

    pub fn add_piece(&mut self, piece: Piece) {
        self.pieces.push(piece);
    }

    pub fn height(&self) -> usize {
        self.pieces.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    pub fn controller(&self) -> Option<Player> {
        if let Some(piece) = self.pieces.last() {
            Some(piece.player)
        } else {
            None
        }
    }

    pub fn is_blocking(&self) -> bool {
        if let Some(piece) = self.pieces.last() {
            piece.is_standing()
        } else {
            false
        }
    }

    pub fn top(&self) -> Option<&Piece> {
        self.pieces.last()
    }

    pub fn take_top(&mut self, count: usize) -> Vec<Piece> {
        let start_idx = self.height().saturating_sub(count);
        self.pieces.drain(start_idx..).collect()
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            write!(f, ".")
        } else {
            let top = self.pieces.last().unwrap();
            let symbol = match (top.player, top.piece_type) {
                (Player::White, PieceType::Flat) => "W",
                (Player::White, PieceType::Standing) => "C",
                (Player::Black, PieceType::Flat) => "B",
                (Player::Black, PieceType::Standing) => "c",
            };
            write!(f, "{}{}", symbol, self.height())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub size: usize,
    pub spaces: Vec<Vec<Stack>>,
    white_pieces: (usize, usize), // (flats, capstones)
    black_pieces: (usize, usize),
}

impl Board {
    pub fn new(size: usize) -> Self {
        let mut spaces = Vec::with_capacity(size);
        for _ in 0..size {
            let mut row = Vec::with_capacity(size);
            for _ in 0..size {
                row.push(Stack::new());
            }
            spaces.push(row);
        }

        // Calculate the number of pieces based on board size
        let (flats, caps) = match size {
            3 => (10, 0),
            4 => (15, 0),
            5 => (21, 1),
            6 => (30, 1),
            _ => (21, 1), // Default to 5x5 rules
        };

        Board {
            size,
            spaces,
            white_pieces: (flats, caps),
            black_pieces: (flats, caps),
        }
    }

    pub fn place_piece(
        &mut self,
        row: usize,
        col: usize,
        player: Player,
        piece_type: PieceType,
    ) -> Result<(), &'static str> {
        // Check if position is within bounds
        if row >= self.size || col >= self.size {
            return Err("Position out of bounds");
        }

        // Check if space is empty
        if !self.spaces[row][col].is_empty() {
            return Err("Space is already occupied");
        }

        // Check if player has enough pieces
        let pieces = match player {
            Player::White => &mut self.white_pieces,
            Player::Black => &mut self.black_pieces,
        };

        match piece_type {
            PieceType::Flat => {
                if pieces.0 == 0 {
                    return Err("No flat stones remaining");
                }
                pieces.0 -= 1;
            }
            PieceType::Standing => {
                if pieces.1 == 0 {
                    return Err("No capstones remaining");
                }
                pieces.1 -= 1;
            }
        }

        // Place the piece
        self.spaces[row][col].add_piece(Piece::new(player, piece_type));
        Ok(())
    }

    pub fn move_stack(
        &mut self,
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
        count: usize,
        direction: Direction,
    ) -> Result<(), &'static str> {
        // Check if positions are within bounds
        if from_row >= self.size
            || from_col >= self.size
            || to_row >= self.size
            || to_col >= self.size
        {
            return Err("Position out of bounds");
        }

        // Check if stack at from position is valid
        let from_stack = &self.spaces[from_row][from_col];
        if from_stack.is_empty() {
            return Err("Source stack is empty");
        }

        // Check if player controls the stack
        let player = if let Some(player) = from_stack.controller() {
            player
        } else {
            return Err("Stack has no controller");
        };

        // Check count is valid
        let stack_height = from_stack.height();
        if count > stack_height || count > self.size {
            return Err("Cannot move that many pieces");
        }

        // Check if move is in a straight line and adjacent
        let (dr, dc) = match direction {
            Direction::North => {
                if from_row <= 0 || to_row != from_row - 1 || to_col != from_col {
                    return Err("Invalid movement - not in a straight line");
                }
                (-1, 0)
            }
            Direction::South => {
                if from_row >= self.size - 1 || to_row != from_row + 1 || to_col != from_col {
                    return Err("Invalid movement - not in a straight line");
                }
                (1, 0)
            }
            Direction::East => {
                if from_col >= self.size - 1 || to_col != from_col + 1 || to_row != from_row {
                    return Err("Invalid movement - not in a straight line");
                }
                (0, 1)
            }
            Direction::West => {
                if from_col <= 0 || to_col != from_col - 1 || to_row != from_row {
                    return Err("Invalid movement - not in a straight line");
                }
                (0, -1)
            }
        };

        // Check if there's a standing stone or capstone in the path
        let mut pieces_to_move = self.spaces[from_row][from_col].take_top(count);
        let mut curr_row = to_row as isize;
        let mut curr_col = to_col as isize;

        // Place the pieces along the path
        while !pieces_to_move.is_empty() {
            if curr_row < 0
                || curr_row >= self.size as isize
                || curr_col < 0
                || curr_col >= self.size as isize
            {
                // Return the pieces and error
                for piece in pieces_to_move.drain(..).rev() {
                    self.spaces[from_row][from_col].add_piece(piece);
                }
                return Err("Movement path out of bounds");
            }

            let curr_r = curr_row as usize;
            let curr_c = curr_col as usize;

            // Check if there's a blocking stone
            if let Some(top) = self.spaces[curr_r][curr_c].top() {
                if top.is_standing() {
                    // Check if we can flatten a wall with a capstone
                    if top.piece_type == PieceType::Flat
                        && pieces_to_move.len() == 1
                        && pieces_to_move[0].piece_type == PieceType::Standing
                    {
                        // Continue - capstone can flatten a wall
                    } else {
                        // Return the pieces and error
                        for piece in pieces_to_move.drain(..).rev() {
                            self.spaces[from_row][from_col].add_piece(piece);
                        }
                        return Err("Cannot move onto a standing stone or capstone");
                    }
                }
            }

            // Place one piece on this space
            let piece = pieces_to_move.remove(0);
            self.spaces[curr_r][curr_c].add_piece(piece);

            // Move to next space
            curr_row += dr;
            curr_col += dc;
        }

        Ok(())
    }

    pub fn is_board_full(&self) -> bool {
        for row in 0..self.size {
            for col in 0..self.size {
                if self.spaces[row][col].is_empty() {
                    return false;
                }
            }
        }
        true
    }

    pub fn check_road_win(&self, player: Player) -> bool {
        // Check for a road win (connected path from one side to the other)
        self.check_horizontal_road(player) || self.check_vertical_road(player)
    }

    fn check_horizontal_road(&self, player: Player) -> bool {
        // Initialize visited array
        let mut visited = vec![vec![false; self.size]; self.size];

        // Check if any piece in the leftmost column can form a road to the rightmost column
        for row in 0..self.size {
            if let Some(stack_player) = self.spaces[row][0].controller() {
                if stack_player == player && self.spaces[row][0].top().unwrap().is_flat() {
                    if self.dfs_road(row, 0, player, &mut visited, true) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn check_vertical_road(&self, player: Player) -> bool {
        // Initialize visited array
        let mut visited = vec![vec![false; self.size]; self.size];

        // Check if any piece in the top row can form a road to the bottom row
        for col in 0..self.size {
            if let Some(stack_player) = self.spaces[0][col].controller() {
                if stack_player == player && self.spaces[0][col].top().unwrap().is_flat() {
                    if self.dfs_road(0, col, player, &mut visited, false) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn dfs_road(
        &self,
        row: usize,
        col: usize,
        player: Player,
        visited: &mut Vec<Vec<bool>>,
        is_horizontal: bool,
    ) -> bool {
        // Mark current cell as visited
        visited[row][col] = true;

        // Check if we've reached the opposite edge
        if (is_horizontal && col == self.size - 1) || (!is_horizontal && row == self.size - 1) {
            return true;
        }

        // Directions: up, right, down, left
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        // Explore all directions
        for (dr, dc) in directions.iter() {
            let new_row = row as isize + dr;
            let new_col = col as isize + dc;

            if new_row >= 0
                && new_row < self.size as isize
                && new_col >= 0
                && new_col < self.size as isize
            {
                let nr = new_row as usize;
                let nc = new_col as usize;

                if !visited[nr][nc]
                    && !self.spaces[nr][nc].is_empty()
                    && self.spaces[nr][nc].controller() == Some(player)
                    && self.spaces[nr][nc].top().unwrap().is_flat()
                {
                    if self.dfs_road(nr, nc, player, visited, is_horizontal) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn remaining_pieces(&self, player: Player) -> (usize, usize) {
        match player {
            Player::White => self.white_pieces,
            Player::Black => self.black_pieces,
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
                .collect::<Vec<String>>()
                .join("")
        )?;
        writeln!(f, "  {}", "-".repeat(self.size * 3 + 1))?;
        for r in 0..self.size {
            write!(f, "{} |", r)?;
            for c in 0..self.size {
                write!(f, " {} ", self.spaces[r][c])?;
            }
            writeln!(f, "|")?;
        }
        writeln!(f, "  {}", "-".repeat(self.size * 3 + 1))?;

        // Print pieces remaining
        writeln!(
            f,
            "White pieces: {} flat, {} capstone",
            self.white_pieces.0, self.white_pieces.1
        )?;
        writeln!(
            f,
            "Black pieces: {} flat, {} capstone",
            self.black_pieces.0, self.black_pieces.1
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone)]
pub enum GameMove {
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

#[derive(Debug)]
pub struct Game {
    pub board: Board,
    pub current_player: Player,
    pub turn_number: usize,
}

impl Game {
    pub fn new(board_size: usize) -> Self {
        Game {
            board: Board::new(board_size),
            current_player: Player::White,
            turn_number: 1,
        }
    }

    pub fn make_move(&mut self, game_move: GameMove) -> Result<(), &'static str> {
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

    pub fn run(&mut self) {
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

            if parts.is_empty() {
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
                    println!("Unknown command. Valid commands are 'place', 'move', or 'quit'.");
                    continue;
                }
            };

            if let Err(msg) = result {
                println!("Error: {}", msg);
                continue;
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

            // Switch player
            self.current_player = match self.current_player {
                Player::White => Player::Black,
                Player::Black => {
                    self.turn_number += 1;
                    Player::White
                }
            };
        }
    }

    pub fn run_human_vs_ai(&mut self, ai_player: Player, difficulty: usize) {
        println!("Welcome to Tak - Human vs AI!");
        println!("Board size: {}", self.board.size);
        println!("AI difficulty: {}", difficulty);

        loop {
            println!(
                "\nTurn {}: {}'s turn",
                self.turn_number, self.current_player
            );
            println!("{}", self.board);

            let result = if self.current_player == ai_player {
                // AI's turn
                println!("AI is thinking...");
                thread::sleep(Duration::from_millis(1000)); // Simulate thinking

                let ai_move = self.ai_choose_move(difficulty);

                // Display the AI's move
                match ai_move {
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

                self.make_move(ai_move)
            } else {
                // Human's turn
                println!(
                    "Enter your move (place row col [flat|cap] or move from_row from_col to_row to_col count):"
                );
                let mut input = String::new();
                io::stdin()
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

            // Switch player
            self.current_player = match self.current_player {
                Player::White => Player::Black,
                Player::Black => {
                    self.turn_number += 1;
                    Player::White
                }
            };
        }
    }

    pub fn run_ai_vs_ai(
        &mut self,
        white_difficulty: usize,
        black_difficulty: usize,
        delay_ms: u64,
    ) {
        println!("Welcome to Tak - AI vs AI!");
        println!("Board size: {}", self.board.size);
        println!("White AI difficulty: {}", white_difficulty);
        println!("Black AI difficulty: {}", black_difficulty);

        loop {
            println!(
                "\nTurn {}: {}'s turn",
                self.turn_number, self.current_player
            );
            println!("{}", self.board);

            // Add delay for better visualization
            if delay_ms > 0 {
                thread::sleep(Duration::from_millis(delay_ms));
            }

            // Choose AI difficulty based on current player
            let difficulty = match self.current_player {
                Player::White => white_difficulty,
                Player::Black => black_difficulty,
            };

            // AI's turn
            println!("AI is thinking...");
            let ai_move = self.ai_choose_move(difficulty);

            // Display the AI's move
            match ai_move {
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

                // Try a simple fallback move
                let mut fallback_successful = false;
                'outer: for row in 0..self.board.size {
                    for col in 0..self.board.size {
                        if self.board.spaces[row][col].is_empty() {
                            let fallback_move = GameMove::Place {
                                row,
                                col,
                                piece_type: PieceType::Flat,
                            };
                            println!(
                                "{} AI attempts fallback move: place flat at ({},{})",
                                self.current_player, row, col
                            );
                            if self.make_move(fallback_move).is_ok() {
                                fallback_successful = true;
                                break 'outer;
                            }
                        }
                    }
                }

                if !fallback_successful {
                    println!("AI could not find a valid move. Game over.");
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

            // Switch player
            self.current_player = match self.current_player {
                Player::White => Player::Black,
                Player::Black => {
                    self.turn_number += 1;
                    Player::White
                }
            };
        }
    }

    fn ai_choose_move(&self, difficulty: usize) -> GameMove {
        let mut rng = rand::thread_rng();
        let board_size = self.board.size;

        // Basic AI logic
        if difficulty == 1 {
            // Easy - Just place random pieces or make random moves
            let place_prob = if self.turn_number <= 2 { 1.0 } else { 0.7 };

            if rng.gen_bool(place_prob) {
                // Try to place a piece randomly
                let mut empty_spaces = Vec::new();
                for row in 0..board_size {
                    for col in 0..board_size {
                        if self.board.spaces[row][col].is_empty() {
                            empty_spaces.push((row, col));
                        }
                    }
                }

                if !empty_spaces.is_empty() {
                    let idx = rng.gen_range(0..empty_spaces.len());
                    let (row, col) = empty_spaces[idx];

                    // Decide piece type (small chance of capstone after first few turns)
                    let use_capstone = rng.gen_bool(0.1)
                        && self.turn_number > 4
                        && self.board.remaining_pieces(self.current_player).1 > 0;

                    let piece_type = if use_capstone {
                        PieceType::Standing
                    } else {
                        PieceType::Flat
                    };

                    return GameMove::Place {
                        row,
                        col,
                        piece_type,
                    };
                }
            }

            // Try to move a stack
            let mut controlled_stacks = Vec::new();
            for row in 0..board_size {
                for col in 0..board_size {
                    if !self.board.spaces[row][col].is_empty()
                        && self.board.spaces[row][col].controller() == Some(self.current_player)
                    {
                        controlled_stacks.push((row, col));
                    }
                }
            }

            if !controlled_stacks.is_empty() {
                let attempts = 5; // Try a few times to find a valid move
                for _ in 0..attempts {
                    let idx = rng.gen_range(0..controlled_stacks.len());
                    let (from_row, from_col) = controlled_stacks[idx];

                    let height = self.board.spaces[from_row][from_col].height();
                    let count = rng.gen_range(1..=height.min(board_size));

                    // Pick a random direction
                    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)]; // right, down, left, up
                    let dir_idx = rng.gen_range(0..4);
                    let (dr, dc) = directions[dir_idx];

                    let to_row = (from_row as isize + dr) as usize;
                    let to_col = (from_col as isize + dc) as usize;

                    // Check if move is valid (in bounds)
                    if to_row < board_size && to_col < board_size {
                        return GameMove::Move {
                            from_row,
                            from_col,
                            to_row,
                            to_col,
                            count,
                        };
                    }
                }
            }
        } else if difficulty >= 2 {
            // Medium/Hard - More strategic play

            // Early game: place in strategic positions
            if self.turn_number <= 2 || rng.gen_bool(0.6) {
                let center = board_size / 2;

                // Prioritize center and surrounding positions
                let mut priority_spots = Vec::new();
                for r in center.saturating_sub(1)..=center.saturating_add(1).min(board_size - 1) {
                    for c in center.saturating_sub(1)..=center.saturating_add(1).min(board_size - 1)
                    {
                        if self.board.spaces[r][c].is_empty() {
                            let priority = if r == center && c == center {
                                3
                            } else if r == center || c == center {
                                2
                            } else {
                                1
                            };
                            for _ in 0..priority {
                                priority_spots.push((r, c));
                            }
                        }
                    }
                }

                // Also consider edge positions
                let mut edge_spots = Vec::new();
                for r in 0..board_size {
                    for c in 0..board_size {
                        if (r == 0 || r == board_size - 1 || c == 0 || c == board_size - 1)
                            && self.board.spaces[r][c].is_empty()
                        {
                            edge_spots.push((r, c));
                        }
                    }
                }

                // Combine and pick a position
                priority_spots.extend(edge_spots);

                if !priority_spots.is_empty() {
                    let idx = rng.gen_range(0..priority_spots.len());
                    let (row, col) = priority_spots[idx];

                    // Decide piece type
                    let use_capstone = difficulty >= 3
                        && rng.gen_bool(0.2)
                        && self.turn_number > 4
                        && self.board.remaining_pieces(self.current_player).1 > 0;

                    let piece_type = if use_capstone {
                        PieceType::Standing
                    } else {
                        PieceType::Flat
                    };

                    return GameMove::Place {
                        row,
                        col,
                        piece_type,
                    };
                }

                // If no strategic spots, try any empty spot
                let mut empty_spots = Vec::new();
                for row in 0..board_size {
                    for col in 0..board_size {
                        if self.board.spaces[row][col].is_empty() {
                            empty_spots.push((row, col));
                        }
                    }
                }

                if !empty_spots.is_empty() {
                    let idx = rng.gen_range(0..empty_spots.len());
                    let (row, col) = empty_spots[idx];
                    return GameMove::Place {
                        row,
                        col,
                        piece_type: PieceType::Flat,
                    };
                }
            }

            // Try to make a strategic stack move

            // First, find all of our controlled stacks
            let mut controlled_stacks = Vec::new();
            for row in 0..board_size {
                for col in 0..board_size {
                    if !self.board.spaces[row][col].is_empty()
                        && self.board.spaces[row][col].controller() == Some(self.current_player)
                    {
                        // Higher score for taller stacks
                        let height = self.board.spaces[row][col].height();
                        for _ in 0..height {
                            controlled_stacks.push((row, col));
                        }
                    }
                }
            }

            if !controlled_stacks.is_empty() {
                let attempts = 10; // Try several times to find a good move
                for _ in 0..attempts {
                    let idx = rng.gen_range(0..controlled_stacks.len());
                    let (from_row, from_col) = controlled_stacks[idx];

                    let height = self.board.spaces[from_row][from_col].height();
                    let max_move = height.min(board_size);

                    // Prefer moving just 1 piece for small stacks, more for large stacks
                    let count = if max_move <= 2 || rng.gen_bool(0.6) {
                        1
                    } else {
                        rng.gen_range(1..=max_move)
                    };

                    // Pick a direction, prioritize capturing opponent pieces or building a road
                    let mut directions = Vec::new();

                    // Check each direction (right, down, left, up)
                    let possible_dirs = [(0, 1), (1, 0), (0, -1), (-1, 0)];

                    for (dr, dc) in possible_dirs.iter() {
                        let to_row = from_row as isize + dr;
                        let to_col = from_col as isize + dc;

                        if to_row >= 0
                            && to_row < board_size as isize
                            && to_col >= 0
                            && to_col < board_size as isize
                        {
                            let tr = to_row as usize;
                            let tc = to_col as usize;

                            // If space is empty or we can overtake it, consider it
                            if self.board.spaces[tr][tc].is_empty()
                                || (self.board.spaces[tr][tc]
                                    .top()
                                    .map_or(false, |p| p.is_flat()))
                            {
                                // Score the move
                                let mut score = 1;

                                // Higher score for capturing opponent's piece
                                if !self.board.spaces[tr][tc].is_empty()
                                    && self.board.spaces[tr][tc].controller()
                                        != Some(self.current_player)
                                {
                                    score += 2;
                                }

                                // Add this direction with its score
                                for _ in 0..score {
                                    directions.push((tr, tc));
                                }
                            }
                        }
                    }

                    if !directions.is_empty() {
                        let idx = rng.gen_range(0..directions.len());
                        let (to_row, to_col) = directions[idx];

                        return GameMove::Move {
                            from_row,
                            from_col,
                            to_row,
                            to_col,
                            count,
                        };
                    }
                }
            }
        }

        // Final fallback - find any valid placement
        for row in 0..board_size {
            for col in 0..board_size {
                if self.board.spaces[row][col].is_empty() {
                    return GameMove::Place {
                        row,
                        col,
                        piece_type: PieceType::Flat,
                    };
                }
            }
        }

        // Ultimate fallback - should never reach here
        GameMove::Place {
            row: 0,
            col: 0,
            piece_type: PieceType::Flat,
        }
    }
}

// Now add the learning_ai module and main function

mod learning_ai;
use learning_ai::{SelfLearningAI, run_human_vs_learning_ai, run_human_vs_trained_ai, train_ai};

fn main() {
    println!("Welcome to Tak Simulator!");
    println!("Game modes:");
    println!("1. Human vs Human");
    println!("2. Human vs AI");
    println!("3. AI vs AI");
    println!("4. Train AI");
    println!("5. Human vs Trained AI");
    println!("Choose a mode (1-5):");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let mode = input.trim().parse::<usize>().unwrap_or(1);

    println!("Enter board size (3-6):");
    input.clear();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let size = input.trim().parse::<usize>().unwrap_or(5);
    let board_size = if size < 3 || size > 6 {
        println!("Invalid board size, using default 5x5");
        5
    } else {
        size
    };

    match mode {
        1 => {
            let mut game = Game::new(board_size);
            game.run();
        }
        2 => {
            println!("Do you want to play as White or Black? (white/black):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            let human_player = match input.trim().to_lowercase().as_str() {
                "black" => Player::Black,
                _ => Player::White, // Default to White
            };

            let ai_player = match human_player {
                Player::White => Player::Black,
                Player::Black => Player::White,
            };

            println!("Enter AI difficulty (1-3):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let ai_difficulty = input.trim().parse::<usize>().unwrap_or(2);

            let mut game = Game::new(board_size);
            game.run_human_vs_ai(ai_player, ai_difficulty);
        }
        3 => {
            println!("Enter White AI difficulty (1-3):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let white_difficulty = input.trim().parse::<usize>().unwrap_or(2);

            println!("Enter Black AI difficulty (1-3):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let black_difficulty = input.trim().parse::<usize>().unwrap_or(2);

            println!("Enter delay between moves in milliseconds (0-5000, 0 for no delay):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let delay = input.trim().parse::<u64>().unwrap_or(1000).min(5000);

            let mut game = Game::new(board_size);
            game.run_ai_vs_ai(white_difficulty, black_difficulty, delay);
        }
        4 => {
            // AI Training mode
            println!("Enter number of training games:");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let num_games = input.trim().parse::<usize>().unwrap_or(1000);

            println!("Enter learning rate (0.01-0.5, recommended 0.1):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let learning_rate = input.trim().parse::<f64>().unwrap_or(0.1).clamp(0.01, 0.5);

            println!("Enter maximum turns per game (10-200):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let max_turns = input.trim().parse::<usize>().unwrap_or(100).clamp(10, 200);

            println!("Show verbose output? (y/n):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            let verbose = input.trim().to_lowercase() == "y";

            // Run training
            let (white_ai, black_ai) = train_ai(
                board_size,
                num_games,
                learning_rate,
                learning_rate,
                max_turns,
                verbose,
            );

            // Ask to save the trained AIs
            println!("\nDo you want to save the trained AIs? (y/n):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if input.trim().to_lowercase() == "y" {
                println!("Enter filename for White AI (e.g. 'white_ai.weights'):");
                input.clear();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                let white_filename = input.trim();
                if !white_filename.is_empty() {
                    if let Err(e) = white_ai.save_weights(white_filename) {
                        println!("Error saving White AI: {}", e);
                    } else {
                        println!("White AI saved to {}", white_filename);
                    }
                }

                println!("Enter filename for Black AI (e.g. 'black_ai.weights'):");
                input.clear();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                let black_filename = input.trim();
                if !black_filename.is_empty() {
                    if let Err(e) = black_ai.save_weights(black_filename) {
                        println!("Error saving Black AI: {}", e);
                    } else {
                        println!("Black AI saved to {}", black_filename);
                    }
                }
            }

            // Ask to play against the trained AI
            println!("\nDo you want to play against the trained AI? (y/n):");
            input.clear();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if input.trim().to_lowercase() == "y" {
                run_human_vs_trained_ai(board_size, white_ai, black_ai);
            }
        }
        5 => {
            // Human vs Trained AI
            println!("Enter filename for AI weights (leave empty for default AI):");
            let mut weights_input = String::new();
            std::io::stdin()
                .read_line(&mut weights_input)
                .expect("Failed to read line");

            let weights_file = weights_input.trim();

            println!("Do you want to play as White or Black? (white/black):");
            let mut player_input = String::new();
            std::io::stdin()
                .read_line(&mut player_input)
                .expect("Failed to read line");

            let human_player = match player_input.trim().to_lowercase().as_str() {
                "black" => Player::Black,
                _ => Player::White, // Default to White
            };

            // Create AI with loaded weights
            let ai_player = match human_player {
                Player::White => Player::Black,
                Player::Black => Player::White,
            };

            let mut ai = SelfLearningAI::new(ai_player, 0.1);

            if !weights_file.is_empty() {
                match ai.load_weights(weights_file) {
                    Ok(_) => println!("Successfully loaded AI weights from {}", weights_file),
                    Err(e) => println!("Error loading weights: {}. Using default AI.", e),
                }
            } else {
                println!("Using default AI settings");
            }

            run_human_vs_learning_ai(board_size, human_player, ai);
        }
        _ => {
            println!("Invalid mode, using Human vs Human");
            let mut game = Game::new(board_size);
            game.run();
        }
    }
}
