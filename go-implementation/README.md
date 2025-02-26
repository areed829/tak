# Tak Game Simulation

A Go implementation of the abstract strategy board game Tak, inspired by Patrick Rothfuss's Kingkiller Chronicle series and designed by James Ernest.

## About Tak

Tak is an elegant two-player abstract strategy game that simulates building and connecting roads. Players place and move pieces called "stones" on a square board with the goal of creating a road connecting opposite sides or controlling the most spaces with flat stones.

## Features

- Complete implementation of Tak rules including:
  - Flat stones, standing stones, and capstones
  - Stack manipulation and movement
  - Support for different board sizes (3x3 up to 8x8)
  - Proper piece allocation based on board size
- Text-based user interface
- Computer opponent with adjustable difficulty
- Move hints for learning
- Support for Portable Tak Notation (PTN)

## Installation

### Prerequisites

- Go (1.13 or later recommended)

### Getting Started

1. Clone the repository or download the source code
2. Navigate to the project directory
3. Build and run the game:

```bash
go build tak-simulation.go
./tak-simulation
```

Or run directly without building:

```bash
go run tak-simulation.go
```

## How to Play

### Gameplay

Tak is played on a square board (usually 5x5). Players take turns placing or moving pieces, with the goal of creating a road (a connected path of their flat stones from one side of the board to the opposite side) or controlling the most spaces.

### Game Setup

- The game will prompt you to select a board size (3-8)
- Choose whether to play against the computer or another human
- If playing against the computer, select your color (white or black)

### Commands

#### Placement Moves

- Place a flat stone: `a1` (any position using letter for column, number for row)
- Place a standing stone: `Sa1` (prefix 'S' followed by position)
- Place a capstone: `Ca1` (prefix 'C' followed by position)

#### Movement Moves

Movement moves use the format: `[count][position][direction][drops]`

- Basic movement: `2a1>` (move 2 pieces from a1 to the right)
  - Direction symbols: `>` (right), `<` (left), `+` (up), `-` (down)
- Stack distribution: `3a1>12` (move 3 pieces from a1 right, dropping 1 in first space, 2 in second)

#### Special Commands

- Hint: `hint` (get a move suggestion from the computer)
- Quit: `quit` (exit the game)

### Win Conditions

1. Road Victory: Create a continuous path of your flat stones or capstones connecting opposite sides of the board
2. Flat Victory: When the board is filled or all pieces are placed, the player with the most flat stones on top wins

## Technical Details

### AI Implementation

The computer opponent uses:

- Minimax algorithm with alpha-beta pruning
- Board evaluation based on:
  - Control of flat spaces
  - Center control
  - Road potential
- Various difficulty levels based on search depth

### Code Structure

- Board representation using 3D slices for stacks
- Move generation and validation
- Game state tracking and evaluation

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Patrick Rothfuss for creating the world where Tak exists
- James Ernest for designing the actual game
- The Tak community for their support and rule clarifications
