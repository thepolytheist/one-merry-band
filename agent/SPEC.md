# Specification for One Merry Band

## Overview
One Merry Band is a Tolkienesque roguelite in which the player will control a band of adventurers seeking to get to the depths of the Romia Mines. The game is played through a terminal interface, with the player controlling the movement and actions of their adventurers while facing off against enemies encountered in the mines. Each level is randomly generated, though special levels related to the story can occasionally be discovered, and these levels do not change.

## Gameplay

### The Goal
The goal of One Merry Band is to get to the lowest level of the Romia Mines. Here, the player will discover one of several possible ending levels depending on which theme they are in for their current run. It's possible that they will have to defeat an eldritch abomination, kill a goblin warlord, or banish a spectral knight from the realm.

### The Band
The titular band is made up of adventurers. When starting a run, the player has a budget of 10 points for assembling their party. The player can choose any combination of adventurers within the budget.

#### Adventurers

##### Fighter (3 points)
The Fighter has average durability and speed. Her specialty is close combat. The Fighter is capable of restoring some health to one adventurer at the end of each level.

The Fighter is represented by the `@` character.

##### Archer (3 points)
The Archer has below-average durability but better-than-average speed. He can attack opponents at range, and opponents are slowed down slightly while within range of the Archer's weapon.

The Archer is represented by the `%` character.

##### Wizard (5 points)
The Wizard has average durability and speed but is capable of tremendous damage at range. Once every few levels the Wizard can attempt to turn an enemy into a chicken, removing them from combat.

The Wizard is represented by the `&` character.

##### Scout (1 point)
The Scout has below-average durability and speed. However, other adventurers do more damage to enemies that are adjacent to a Scout. Additionally, when traveling between levels, the Scout allows the player to preview the enemies and any special events in that level before going to it.

The Scout is represented by the `$` character.

### Rounds
Play happens in rounds. Whichever adventurer or enemy has the highest speed goes first (with a tie granted to the adventurer), and then the next-fastest adventurer or enemy goes, and so on and so forth.

During the round, the adventurer or enemy may use their action points to move or attack.

## Technical Details
One Merry Band is a Rust project using [Ratatui](https://github.com/ratatui/ratatui). The upper three-quarters of the screen display the current level, while the bottom of the screen displays the menu, adventurer statistics, etc.