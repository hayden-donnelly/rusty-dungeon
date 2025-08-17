# Rusty Dungeon
This is a small game I made for fun. It was programmed for Linux with Rust and inline x86-64 Assembly. It doesn't use the C Runtime or the Rust Standard Library. 

## Requirements
- A computer with x86-64 Linux installed
- A rust compiler (rustc 1.89.0-nightly other versions might not work)

## Building and Running the Game
0. Execute `nix develop` to open a Nix dev shell with the required rustc version installed (you can skip this if you already have it or you want to try your luck with a different version of rustc).

1. Execute `rustc -C panic=abort -C link-arg=-nostartfiles main.rs` to compile and link the game.

2. Execute `./main` to run the game.

## Controls
- Arrow keys to move up, down, left, and right
- Space key to dismiss on-screen messages
- Q key to quit game

## Game Screen
```
#########............###################
#########............###################
#########............###################
#########...............################
#########...............################
#########...............################
#########...............################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.#####.S@...################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.#####......################
##########.#.########.##################
You must find the key!
```
