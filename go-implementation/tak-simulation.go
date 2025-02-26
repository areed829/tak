package main

import (
	"errors"
	"fmt"
	"math/rand"
	"strconv"
	"strings"
	"time"
)

// Constants for piece types and players
const (
	Empty = iota
	FlatWhite
	FlatBlack
	StandingWhite
	StandingBlack
	CapstoneWhite
	CapstoneBlack
)

// Player constants
const (
	White = 1
	Black = 2
)

// Game represents the Tak game state
type Game struct {
	Size           int
	Board          [][][]int // [x][y][stack position] - stack grows upward
	CurrentPlayer  int
	WhitePieces    int
	BlackPieces    int
	WhiteCapstones int
	BlackCapstones int
	MovesHistory   []string
}

// NewGame creates a new Tak game with the specified board size
func NewGame(size int) *Game {
	// Initialize empty board
	board := make([][][]int, size)
	for i := range board {
		board[i] = make([][]int, size)
		for j := range board[i] {
			board[i][j] = make([]int, 0)
		}
	}

	// Calculate initial pieces based on board size
	flatPieces := calculateFlatPieces(size)
	capstones := calculateCapstones(size)

	return &Game{
		Size:           size,
		Board:          board,
		CurrentPlayer:  White, // White goes first by default
		WhitePieces:    flatPieces,
		BlackPieces:    flatPieces,
		WhiteCapstones: capstones,
		BlackCapstones: capstones,
		MovesHistory:   []string{},
	}
}

// Calculate number of flat/standing pieces based on board size
func calculateFlatPieces(size int) int {
	switch size {
	case 3:
		return 10
	case 4:
		return 15
	case 5:
		return 21
	case 6:
		return 30
	case 7:
		return 40
	case 8:
		return 50
	default:
		return 21 // Default to 5x5 game
	}
}

// Calculate number of capstones based on board size
func calculateCapstones(size int) int {
	if size < 5 {
		return 0
	}
	if size < 7 {
		return 1
	}
	return 2
}

// PlayMove processes a move in Portable Tak Notation (PTN)
func (g *Game) PlayMove(moveStr string) error {
	moveStr = strings.TrimSpace(moveStr)

	// Check for placement move (e.g., "a1", "Sa1", "Ca1")
	if len(moveStr) >= 2 && !strings.Contains(moveStr, "-") && !strings.Contains(moveStr, "+") {
		return g.handlePlacementMove(moveStr)
	}

	// Check for movement move (e.g., "2a2>", "3a1+21", etc.)
	if len(moveStr) >= 3 && (strings.Contains(moveStr, ">") || strings.Contains(moveStr, "<") ||
		strings.Contains(moveStr, "+") || strings.Contains(moveStr, "-")) {
		return g.handleMovementMove(moveStr)
	}

	return errors.New("invalid move format")
}

// HandlePlacementMove processes a placement move
func (g *Game) handlePlacementMove(moveStr string) error {
	var x, y int
	var pieceType int

	// Default is flat stone
	pieceType = FlatWhite
	if g.CurrentPlayer == Black {
		pieceType = FlatBlack
	}

	// Check if it's a standing stone or capstone
	if strings.HasPrefix(moveStr, "S") {
		if g.CurrentPlayer == White {
			pieceType = StandingWhite
		} else {
			pieceType = StandingBlack
		}
		moveStr = moveStr[1:]
	} else if strings.HasPrefix(moveStr, "C") {
		if g.CurrentPlayer == White {
			if g.WhiteCapstones <= 0 {
				return errors.New("no white capstones remaining")
			}
			pieceType = CapstoneWhite
			g.WhiteCapstones--
		} else {
			if g.BlackCapstones <= 0 {
				return errors.New("no black capstones remaining")
			}
			pieceType = CapstoneBlack
			g.BlackCapstones--
		}
		moveStr = moveStr[1:]
	}

	// Parse coordinates
	if len(moveStr) < 2 {
		return errors.New("invalid move format")
	}

	// Convert letter to x-coordinate (a=0, b=1, etc.)
	x = int(moveStr[0] - 'a')

	// Convert number to y-coordinate (1=0, 2=1, etc.)
	yVal, err := strconv.Atoi(moveStr[1:])
	if err != nil {
		return errors.New("invalid coordinate")
	}
	y = yVal - 1

	// Validate coordinates
	if x < 0 || x >= g.Size || y < 0 || y >= g.Size {
		return errors.New("coordinates out of bounds")
	}

	// Check if there's already a piece at this position
	if len(g.Board[x][y]) > 0 {
		return errors.New("position already occupied")
	}

	// Update remaining pieces count
	if pieceType == FlatWhite || pieceType == StandingWhite {
		if g.WhitePieces <= 0 {
			return errors.New("no white pieces remaining")
		}
		g.WhitePieces--
	} else if pieceType == FlatBlack || pieceType == StandingBlack {
		if g.BlackPieces <= 0 {
			return errors.New("no black pieces remaining")
		}
		g.BlackPieces--
	}

	// Place the piece
	g.Board[x][y] = append(g.Board[x][y], pieceType)

	// Record the move
	g.MovesHistory = append(g.MovesHistory, moveStr)

	// Switch player
	g.switchPlayer()

	return nil
}

// HandleMovementMove processes a movement move
func (g *Game) handleMovementMove(moveStr string) error {
	// Example: "3a2>"
	// This means "take 3 pieces from a2 and move them right"

	// Parse the count, position, and direction
	var count, x, y int
	var direction rune

	// Find direction
	for _, dir := range []rune{'>', '<', '+', '-'} {
		if idx := strings.IndexRune(moveStr, dir); idx != -1 {
			direction = dir

			// Parse the count
			countStr := moveStr[:idx-2]
			var err error
			count, err = strconv.Atoi(countStr)
			if err != nil {
				return errors.New("invalid count")
			}

			// Parse position
			posStr := moveStr[idx-2 : idx]
			if len(posStr) != 2 {
				return errors.New("invalid position format")
			}

			// Convert letter to x-coordinate (a=0, b=1, etc.)
			x = int(posStr[0] - 'a')

			// Convert number to y-coordinate (1=0, 2=1, etc.)
			yVal, err := strconv.Atoi(posStr[1:2])
			if err != nil {
				return errors.New("invalid coordinate")
			}
			y = yVal - 1

			break
		}
	}

	// Validate coordinates
	if x < 0 || x >= g.Size || y < 0 || y >= g.Size {
		return errors.New("coordinates out of bounds")
	}

	// Check if there are enough pieces to move
	if len(g.Board[x][y]) < count {
		return errors.New("not enough pieces to move")
	}

	// Check if the top piece belongs to the current player
	topPiece := g.Board[x][y][len(g.Board[x][y])-1]
	if (g.CurrentPlayer == White && (topPiece == FlatBlack || topPiece == StandingBlack || topPiece == CapstoneBlack)) ||
		(g.CurrentPlayer == Black && (topPiece == FlatWhite || topPiece == StandingWhite || topPiece == CapstoneWhite)) {
		return errors.New("cannot move opponent's pieces")
	}

	// Initialize destination coordinates
	destX, destY := x, y
	var drops []int

	// Parse the drops if provided (e.g., "3a1+12" = drop 1, then 2)
	dropsStr := moveStr[strings.IndexRune(moveStr, direction)+1:]
	if dropsStr != "" {
		for _, c := range dropsStr {
			drop, err := strconv.Atoi(string(c))
			if err != nil {
				return errors.New("invalid drop count")
			}
			drops = append(drops, drop)
		}
	} else {
		// If no drops specified, drop all at the final destination
		drops = []int{count}
	}

	// Ensure the sum of drops equals the count
	sum := 0
	for _, drop := range drops {
		sum += drop
	}
	if sum != count {
		return errors.New("sum of drops does not match count")
	}

	// Move the pieces based on direction
	stack := g.Board[x][y][len(g.Board[x][y])-count:]
	g.Board[x][y] = g.Board[x][y][:len(g.Board[x][y])-count]

	// Move along the path and drop pieces
	for _, drop := range drops {
		// Update destination based on direction
		switch direction {
		case '>': // Right
			destX++
		case '<': // Left
			destX--
		case '+': // Up
			destY++
		case '-': // Down
			destY--
		}

		// Validate the destination
		if destX < 0 || destX >= g.Size || destY < 0 || destY >= g.Size {
			return errors.New("movement out of bounds")
		}

		// Check if there's a standing stone or capstone in the way
		if len(g.Board[destX][destY]) > 0 {
			topDestPiece := g.Board[destX][destY][len(g.Board[destX][destY])-1]

			// Cannot move onto a capstone
			if topDestPiece == CapstoneWhite || topDestPiece == CapstoneBlack {
				return errors.New("cannot move onto a capstone")
			}

			// Check if we're moving a capstone onto a standing stone (flatten it)
			if topDestPiece == StandingWhite || topDestPiece == StandingBlack {
				if len(stack) > 0 && (stack[0] == CapstoneWhite || stack[0] == CapstoneBlack) {
					// Flatten the standing stone
					if topDestPiece == StandingWhite {
						g.Board[destX][destY][len(g.Board[destX][destY])-1] = FlatWhite
					} else {
						g.Board[destX][destY][len(g.Board[destX][destY])-1] = FlatBlack
					}
				} else {
					return errors.New("cannot move onto a standing stone without a capstone")
				}
			}
		}

		// Drop pieces
		dropPieces := stack[:drop]
		stack = stack[drop:]
		g.Board[destX][destY] = append(g.Board[destX][destY], dropPieces...)
	}

	// Record the move
	g.MovesHistory = append(g.MovesHistory, moveStr)

	// Switch player
	g.switchPlayer()

	return nil
}

// Switch to the next player
func (g *Game) switchPlayer() {
	if g.CurrentPlayer == White {
		g.CurrentPlayer = Black
	} else {
		g.CurrentPlayer = White
	}
}

// Check if the game is over and return the winner (0 if not over)
func (g *Game) CheckWinner() (int, string) {
	// Check for road win (White)
	if g.hasRoad(White) {
		return White, "Road"
	}

	// Check for road win (Black)
	if g.hasRoad(Black) {
		return Black, "Road"
	}

	// Check for flat win (board is full)
	isFull := true
	for i := 0; i < g.Size; i++ {
		for j := 0; j < g.Size; j++ {
			if len(g.Board[i][j]) == 0 {
				isFull = false
				break
			}
		}
		if !isFull {
			break
		}
	}

	if isFull {
		// Count flat stones
		whiteFlats, blackFlats := g.countFlats()
		if whiteFlats > blackFlats {
			return White, "Flat"
		} else if blackFlats > whiteFlats {
			return Black, "Flat"
		} else {
			return 0, "Draw" // Draw
		}
	}

	return 0, "" // Game not over
}

// Count flat stones for each player
func (g *Game) countFlats() (int, int) {
	whiteFlats, blackFlats := 0, 0

	for i := 0; i < g.Size; i++ {
		for j := 0; j < g.Size; j++ {
			if len(g.Board[i][j]) > 0 {
				topPiece := g.Board[i][j][len(g.Board[i][j])-1]
				if topPiece == FlatWhite || topPiece == CapstoneWhite {
					whiteFlats++
				} else if topPiece == FlatBlack || topPiece == CapstoneBlack {
					blackFlats++
				}
			}
		}
	}

	return whiteFlats, blackFlats
}

// Check if the player has a road
func (g *Game) hasRoad(player int) bool {
	// For horizontal connections
	for i := 0; i < g.Size; i++ {
		visited := make([][]bool, g.Size)
		for j := range visited {
			visited[j] = make([]bool, g.Size)
		}

		// Start DFS from left edge
		for j := 0; j < g.Size; j++ {
			if g.isPlayerFlat(0, j, player) {
				if g.dfsRoad(0, j, player, visited, true, false) {
					return true
				}
			}
		}
	}

	// For vertical connections
	for i := 0; i < g.Size; i++ {
		visited := make([][]bool, g.Size)
		for j := range visited {
			visited[j] = make([]bool, g.Size)
		}

		// Start DFS from bottom edge
		for j := 0; j < g.Size; j++ {
			if g.isPlayerFlat(j, 0, player) {
				if g.dfsRoad(j, 0, player, visited, false, true) {
					return true
				}
			}
		}
	}

	return false
}

// DFS to find a road
func (g *Game) dfsRoad(x, y int, player int, visited [][]bool, horizontal, vertical bool) bool {
	// Check if we've reached the opposite edge
	if (horizontal && x == g.Size-1) || (vertical && y == g.Size-1) {
		return true
	}

	// Mark as visited
	visited[x][y] = true

	// Check all four directions
	directions := [][2]int{{1, 0}, {-1, 0}, {0, 1}, {0, -1}}
	for _, dir := range directions {
		nx, ny := x+dir[0], y+dir[1]

		// Check bounds
		if nx < 0 || nx >= g.Size || ny < 0 || ny >= g.Size {
			continue
		}

		// Check if not visited and is a player's flat
		if !visited[nx][ny] && g.isPlayerFlat(nx, ny, player) {
			if g.dfsRoad(nx, ny, player, visited, horizontal, vertical) {
				return true
			}
		}
	}

	return false
}

// Check if the position has a flat stone or capstone of the player
func (g *Game) isPlayerFlat(x, y int, player int) bool {
	if len(g.Board[x][y]) == 0 {
		return false
	}

	topPiece := g.Board[x][y][len(g.Board[x][y])-1]
	if player == White {
		return topPiece == FlatWhite || topPiece == CapstoneWhite
	} else {
		return topPiece == FlatBlack || topPiece == CapstoneBlack
	}
}

// PrintBoard displays the current board state
func (g *Game) PrintBoard() {
	fmt.Println("  " + strings.Repeat("--", g.Size+1))

	for y := g.Size - 1; y >= 0; y-- {
		fmt.Printf("%d |", y+1)

		for x := 0; x < g.Size; x++ {
			if len(g.Board[x][y]) == 0 {
				fmt.Print(" .")
			} else {
				topPiece := g.Board[x][y][len(g.Board[x][y])-1]
				switch topPiece {
				case FlatWhite:
					fmt.Print(" W")
				case FlatBlack:
					fmt.Print(" B")
				case StandingWhite:
					fmt.Print(" Sw")
				case StandingBlack:
					fmt.Print(" Sb")
				case CapstoneWhite:
					fmt.Print(" Cw")
				case CapstoneBlack:
					fmt.Print(" Cb")
				}
			}
		}

		fmt.Print(" |\n")
	}

	fmt.Print("  ")
	for x := 0; x < g.Size; x++ {
		fmt.Printf(" %c", 'a'+x)
	}
	fmt.Println("\n  " + strings.Repeat("--", g.Size+1))

	// Game info
	fmt.Printf("White: %d flat pieces, %d capstones\n", g.WhitePieces, g.WhiteCapstones)
	fmt.Printf("Black: %d flat pieces, %d capstones\n", g.BlackPieces, g.BlackCapstones)
	fmt.Printf("Current player: %s\n", g.currentPlayerString())
}

// Helper function to return the current player as a string
func (g *Game) currentPlayerString() string {
	if g.CurrentPlayer == White {
		return "White"
	}
	return "Black"
}

// AI functions for computer opponent

// MoveGenerator generates all possible legal moves for the current player
func (g *Game) MoveGenerator() []string {
	moves := []string{}

	// Player info
	playerPieces := g.WhitePieces
	playerCaps := g.WhiteCapstones
	if g.CurrentPlayer == Black {
		playerPieces = g.BlackPieces
		playerCaps = g.BlackCapstones
	}

	// 1. Generate placement moves
	if playerPieces > 0 || playerCaps > 0 {
		for x := 0; x < g.Size; x++ {
			for y := 0; y < g.Size; y++ {
				// Skip occupied squares
				if len(g.Board[x][y]) > 0 {
					continue
				}

				// Regular flat stone placement
				if playerPieces > 0 {
					moves = append(moves, fmt.Sprintf("%c%d", 'a'+x, y+1))

					// Standing stone placement
					moves = append(moves, fmt.Sprintf("S%c%d", 'a'+x, y+1))
				}

				// Capstone placement
				if playerCaps > 0 {
					moves = append(moves, fmt.Sprintf("C%c%d", 'a'+x, y+1))
				}
			}
		}
	}

	// 2. Generate movement moves
	for x := 0; x < g.Size; x++ {
		for y := 0; y < g.Size; y++ {
			// Skip empty squares
			if len(g.Board[x][y]) == 0 {
				continue
			}

			// Check if top piece belongs to the current player
			topPiece := g.Board[x][y][len(g.Board[x][y])-1]
			if (g.CurrentPlayer == White && (topPiece == FlatBlack || topPiece == StandingBlack || topPiece == CapstoneBlack)) ||
				(g.CurrentPlayer == Black && (topPiece == FlatWhite || topPiece == StandingWhite || topPiece == CapstoneWhite)) {
				continue
			}

			// Generate stack movements
			maxCarry := len(g.Board[x][y])
			if maxCarry > g.Size {
				maxCarry = g.Size
			}

			for carry := 1; carry <= maxCarry; carry++ {
				// For each cardinal direction
				directions := []struct {
					dx, dy  int
					dirChar rune
				}{
					{1, 0, '>'},  // Right
					{-1, 0, '<'}, // Left
					{0, 1, '+'},  // Up
					{0, -1, '-'}, // Down
				}

				for _, dir := range directions {
					// Generate simple drops (all at once)
					nx, ny := x+dir.dx, y+dir.dy

					// Check if in bounds
					if nx < 0 || nx >= g.Size || ny < 0 || ny >= g.Size {
						continue
					}

					// Check if we can move onto the destination
					if len(g.Board[nx][ny]) > 0 {
						topDestPiece := g.Board[nx][ny][len(g.Board[nx][ny])-1]

						// Cannot move onto a capstone
						if topDestPiece == CapstoneWhite || topDestPiece == CapstoneBlack {
							continue
						}

						// Can only move onto a standing stone with a capstone
						if topDestPiece == StandingWhite || topDestPiece == StandingBlack {
							if carry != 1 ||
								((topPiece != CapstoneWhite) && (topPiece != CapstoneBlack)) {
								continue
							}
						}
					}

					// Add the move
					moves = append(moves, fmt.Sprintf("%d%c%d%c", carry, 'a'+x, y+1, dir.dirChar))

					// For larger stacks, generate more complex drop patterns
					if carry > 1 {
						// Generate all possible drop combinations
						// For simplicity, we'll just add a few basic patterns
						if carry == 2 {
							moves = append(moves, fmt.Sprintf("%d%c%d%c%d", carry, 'a'+x, y+1, dir.dirChar, 1))
						} else if carry == 3 {
							moves = append(moves, fmt.Sprintf("%d%c%d%c%d%d", carry, 'a'+x, y+1, dir.dirChar, 1, 2))
							moves = append(moves, fmt.Sprintf("%d%c%d%c%d%d", carry, 'a'+x, y+1, dir.dirChar, 2, 1))
						}
					}
				}
			}
		}
	}

	return moves
}

// EvaluateBoard assigns a score to the current board state from current player's perspective
func (g *Game) EvaluateBoard() int {
	// This is a simple evaluation function
	// Positive score is good for current player, negative is bad

	// Count controlled squares (flat stones and capstones)
	currentPlayerFlats := 0
	opponentFlats := 0

	// Count center control (pieces in center squares are more valuable)
	currentPlayerCenter := 0
	opponentCenter := 0

	// Define center region
	centerMin := g.Size / 3
	centerMax := g.Size - centerMin - 1

	for x := 0; x < g.Size; x++ {
		for y := 0; y < g.Size; y++ {
			if len(g.Board[x][y]) == 0 {
				continue
			}

			topPiece := g.Board[x][y][len(g.Board[x][y])-1]
			isCurrentPlayer := false

			if g.CurrentPlayer == White {
				isCurrentPlayer = (topPiece == FlatWhite || topPiece == CapstoneWhite)
			} else {
				isCurrentPlayer = (topPiece == FlatBlack || topPiece == CapstoneBlack)
			}

			// Count flats and capstones
			if isCurrentPlayer {
				currentPlayerFlats++
				if x >= centerMin && x <= centerMax && y >= centerMin && y <= centerMax {
					currentPlayerCenter++
				}
			} else if topPiece != StandingWhite && topPiece != StandingBlack {
				opponentFlats++
				if x >= centerMin && x <= centerMax && y >= centerMin && y <= centerMax {
					opponentCenter++
				}
			}
		}
	}

	// Check road potential (very simplified)
	currentPlayerRoadPotential := g.evaluateRoadPotential(g.CurrentPlayer)
	opponentRoadPotential := g.evaluateRoadPotential(3 - g.CurrentPlayer) // 3-player gives the opponent

	// Check for actual roads (immediate win/loss)
	if g.hasRoad(g.CurrentPlayer) {
		return 10000 // Current player has a road
	}
	if g.hasRoad(3 - g.CurrentPlayer) {
		return -10000 // Opponent has a road
	}

	// Calculate final score
	score := (currentPlayerFlats-opponentFlats)*10 +
		(currentPlayerCenter-opponentCenter)*15 +
		(currentPlayerRoadPotential-opponentRoadPotential)*20

	return score
}

// EvaluateRoadPotential estimates how close a player is to making a road
func (g *Game) evaluateRoadPotential(player int) int {
	horizontalConnections := 0
	verticalConnections := 0

	// Check horizontal connections
	for y := 0; y < g.Size; y++ {
		currentRun := 0
		for x := 0; x < g.Size; x++ {
			if g.isPlayerFlat(x, y, player) {
				currentRun++
			} else {
				if currentRun > 1 {
					horizontalConnections += currentRun
				}
				currentRun = 0
			}
		}
		if currentRun > 1 {
			horizontalConnections += currentRun
		}
	}

	// Check vertical connections
	for x := 0; x < g.Size; x++ {
		currentRun := 0
		for y := 0; y < g.Size; y++ {
			if g.isPlayerFlat(x, y, player) {
				currentRun++
			} else {
				if currentRun > 1 {
					verticalConnections += currentRun
				}
				currentRun = 0
			}
		}
		if currentRun > 1 {
			verticalConnections += currentRun
		}
	}

	return horizontalConnections + verticalConnections
}

// CloneGame creates a deep copy of the game
func (g *Game) CloneGame() *Game {
	newGame := NewGame(g.Size)

	// Copy the board
	for x := 0; x < g.Size; x++ {
		for y := 0; y < g.Size; y++ {
			newGame.Board[x][y] = make([]int, len(g.Board[x][y]))
			copy(newGame.Board[x][y], g.Board[x][y])
		}
	}

	// Copy game state
	newGame.CurrentPlayer = g.CurrentPlayer
	newGame.WhitePieces = g.WhitePieces
	newGame.BlackPieces = g.BlackPieces
	newGame.WhiteCapstones = g.WhiteCapstones
	newGame.BlackCapstones = g.BlackCapstones

	// Copy moves history
	newGame.MovesHistory = make([]string, len(g.MovesHistory))
	copy(newGame.MovesHistory, g.MovesHistory)

	return newGame
}

// Minimax algorithm with alpha-beta pruning
func (g *Game) Minimax(depth int, alpha, beta int, maximizingPlayer bool) (int, string) {
	// Check for terminal state
	winner, _ := g.CheckWinner()
	if winner != 0 {
		if winner == g.CurrentPlayer {
			return 10000, ""
		} else {
			return -10000, ""
		}
	}

	// If we've reached maximum depth, evaluate the board
	if depth == 0 {
		return g.EvaluateBoard(), ""
	}

	// Generate all possible moves
	moves := g.MoveGenerator()
	if len(moves) == 0 {
		return g.EvaluateBoard(), ""
	}

	var bestMove string

	if maximizingPlayer {
		maxEval := -100000
		for _, move := range moves {
			// Clone the game and make the move
			clonedGame := g.CloneGame()
			err := clonedGame.PlayMove(move)
			if err != nil {
				continue
			}

			// Recursive evaluation
			eval, _ := clonedGame.Minimax(depth-1, alpha, beta, false)

			if eval > maxEval {
				maxEval = eval
				bestMove = move
			}

			alpha = max(alpha, eval)
			if beta <= alpha {
				break
			}
		}
		return maxEval, bestMove
	} else {
		minEval := 100000
		for _, move := range moves {
			// Clone the game and make the move
			clonedGame := g.CloneGame()
			err := clonedGame.PlayMove(move)
			if err != nil {
				continue
			}

			// Recursive evaluation
			eval, _ := clonedGame.Minimax(depth-1, alpha, beta, true)

			if eval < minEval {
				minEval = eval
				bestMove = move
			}

			beta = min(beta, eval)
			if beta <= alpha {
				break
			}
		}
		return minEval, bestMove
	}
}

// Helper functions for min/max
func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}

// GetComputerMove calculates the best move for the computer
func (g *Game) GetComputerMove() string {
	// For first few moves, use simpler strategy or randomness for variety
	if len(g.MovesHistory) < 2 {
		moves := g.MoveGenerator()
		rand.Seed(time.Now().UnixNano())

		// Prefer center positions in early game
		centerMoves := []string{}
		centerMin := g.Size / 3
		centerMax := g.Size - centerMin - 1

		for _, move := range moves {
			if len(move) >= 2 && !strings.ContainsAny(move, "SC") {
				x := int(move[0] - 'a')
				y, err := strconv.Atoi(move[1:2])
				if err != nil {
					continue // Skip if we can't parse the coordinate
				}
				y = y - 1 // Adjust for 0-based indexing

				if x >= centerMin && x <= centerMax && y >= centerMin && y <= centerMax {
					centerMoves = append(centerMoves, move)
				}
			}
		}

		if len(centerMoves) > 0 {
			return centerMoves[rand.Intn(len(centerMoves))]
		}

		return moves[rand.Intn(len(moves))]
	}

	// Use minimax for main game
	searchDepth := 2 // Adjust based on desired difficulty
	_, bestMove := g.Minimax(searchDepth, -100000, 100000, true)
	return bestMove
}

// Main function to demonstrate and test the game
func main() {
	fmt.Println("Tak Game Simulation")
	fmt.Println("-------------------")

	// Initialize random seed
	rand.Seed(time.Now().UnixNano())

	// Get board size
	var size int
	fmt.Print("Enter board size (3-8): ")
	fmt.Scanln(&size)
	if size < 3 || size > 8 {
		size = 5 // Default to 5x5 if invalid input
	}

	// Get game mode
	var playComputer bool
	var playerColor int
	fmt.Print("Play against computer? (y/n): ")
	var response string
	fmt.Scanln(&response)
	playComputer = strings.ToLower(response) == "y"

	if playComputer {
		fmt.Print("Choose your color (w/b): ")
		fmt.Scanln(&response)
		playerColor = Black // Default to black
		if strings.ToLower(response) == "w" {
			playerColor = White
		}
	}

	// Create a new game
	game := NewGame(size)

	// Print initial board
	game.PrintBoard()

	// Game loop
	for {
		// Check for computer move
		if playComputer && game.CurrentPlayer != playerColor {
			fmt.Println("\nComputer is thinking...")
			moveStr := game.GetComputerMove()
			fmt.Printf("Computer plays: %s\n", moveStr)

			err := game.PlayMove(moveStr)
			if err != nil {
				fmt.Printf("Error in computer move: %s\n", err)
				// Try a random move if there's an error
				moves := game.MoveGenerator()
				if len(moves) > 0 {
					moveStr = moves[rand.Intn(len(moves))]
					err = game.PlayMove(moveStr)
					if err != nil {
						fmt.Printf("Computer couldn't make a valid move\n")
						break
					}
				} else {
					fmt.Printf("No valid moves available\n")
					break
				}
			}
		} else {
			// Get move from user
			var moveStr string
			fmt.Printf("\nEnter move for %s (or 'quit' to exit, 'hint' for suggestion): ", game.currentPlayerString())
			fmt.Scanln(&moveStr)

			if strings.ToLower(moveStr) == "quit" {
				break
			}

			// Provide a hint
			if strings.ToLower(moveStr) == "hint" {
				hint := game.GetComputerMove()
				fmt.Printf("Suggested move: %s\n", hint)
				continue
			}

			// Process the move
			err := game.PlayMove(moveStr)
			if err != nil {
				fmt.Printf("Error: %s\n", err)
				continue
			}
		}

		// Print the board
		game.PrintBoard()

		// Check for winner
		winner, winType := game.CheckWinner()
		if winner != 0 {
			if winner == White {
				fmt.Printf("\nWhite wins by %s!\n", winType)
			} else {
				fmt.Printf("\nBlack wins by %s!\n", winType)
			}

			if playComputer {
				if winner == playerColor {
					fmt.Println("Congratulations! You won!")
				} else {
					fmt.Println("The computer won. Better luck next time!")
				}
			}

			break
		}
	}
}
