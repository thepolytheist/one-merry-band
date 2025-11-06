# One Merry Band

A Tolkienesque roguelite terminal game where you control a band of adventurers exploring the depths of the Romia Mines.

## Overview

One Merry Band is a turn-based tactical roguelite built with Rust using the Ratatui TUI framework and a Specs-based Entity Component System (ECS) architecture.

## Building and Running

### Prerequisites

- Rust 1.70 or later
- A terminal that supports ANSI escape codes

### Build

```bash
cd game
cargo build --release
```

### Run

```bash
cargo run --release
```

Or after building:

```bash
./target/release/one_merry_band
```

### Tests

```bash
cargo test
```

## How to Play

### Party Selection

When you start the game, you'll be presented with a party selection screen. You have 10 points to spend on adventurers:

- **Fighter (@)** - 3 points
  - Average durability and speed
  - Close combat specialist
  - HP: 12, Speed: 10, Damage: 10

- **Archer (%)** - 3 points
  - Below-average durability, better speed
  - Ranged attacks
  - HP: 8, Speed: 12, Damage: 8

- **Wizard (&)** - 5 points
  - Average stats, tremendous ranged damage
  - HP: 10, Speed: 10, Damage: 15

- **Scout ($)** - 1 point
  - Below-average stats
  - Enhances other adventurers
  - HP: 8, Speed: 8, Damage: 5

**Party Selection Controls:**
- `↑/↓` - Navigate adventurer list
- `Space/Enter` - Add selected adventurer to party
- `Backspace` - Remove last adventurer from party
- `C` - Confirm party and start game
- `Q` - Quit

### Gameplay

**Movement Controls:**
- `↑/↓/←/→` or `k/j/h/l` (Vim keys) - Move your adventurer
- `Space` - Pass turn
- `Q` - Quit game

**Combat:**
- Move into an enemy to attack (costs 2 action points)
- Each entity gets 2 action points per turn
- Movement costs 1 action point
- Attacking costs 2 action points

**Turn Order:**
- Entities act in order of their speed stat (highest first)
- When all entities have used their action points, a new round begins

**Victory Conditions:**
- Defeat all enemies to win
- If all your adventurers die, it's game over

### Enemies

- **Giant Rat (r)** - Fast but weak (HP: 5, Speed: 12, Damage: 4)
- **Goblin (g)** - Balanced enemy (HP: 8, Speed: 10, Damage: 6)
- **Orc (o)** - Strong and slow (HP: 15, Speed: 8, Damage: 10)
- **Ogre (O)** - Very strong boss (HP: 25, Speed: 6, Damage: 15) *(not yet in prototype)*

## Architecture

The game uses a clean ECS architecture with the following organization:

### Modules

- **components.rs** - ECS components (Position, Stats, ActionPoints, etc.)
- **systems.rs** - Game systems (Movement, Combat, Turn Order)
- **game.rs** - Main game loop and state management
- **level.rs** - Level generation and tile management
- **ui.rs** - Terminal UI rendering with Ratatui
- **main.rs** - Entry point and party selection

### Key Components

- `Position` - Entity location on the map
- `Stats` - Health, speed, and damage values
- `ActionPoints` - Turn-based action economy
- `Adventurer` - Player-controlled character marker
- `Enemy` - AI-controlled enemy marker
- `Renderable` - Display character for rendering

### Systems

- `MovementSystem` - Handles entity movement with collision detection
- `CombatSystem` - Processes melee attacks
- `TurnOrderSystem` - Manages turn order based on speed
- AI system for enemy behavior (integrated into game loop)

## Current Features (Prototype v0.1.0)

- ✅ Party selection with point-buy system
- ✅ Procedurally generated level
- ✅ Turn-based movement system
- ✅ Combat system with melee attacks
- ✅ Action point economy (2 AP per turn)
- ✅ Speed-based turn order
- ✅ Multiple enemy types with AI
- ✅ Health tracking and death
- ✅ Victory and game over conditions
- ✅ Real-time UI updates
- ✅ Comprehensive test coverage

## Future Enhancements

The spec includes many features not yet implemented in this prototype:

- Special abilities (Fighter healing, Wizard polymorph, Archer slowing)
- Multiple levels with progression
- Level preview system (Scout ability)
- Story-based special levels
- Boss encounters
- More sophisticated AI
- Ranged combat
- Inventory system
- Save/load functionality

## License

This is a prototype implementation for demonstration purposes.
