# Sokobauto

analysis tools built around sokoban puzzles to enable easier design or possible autogeneration of interesting puzzles. Also a forde directed graph renderer written in bevy.

Inspired by https://www.youtube.com/watch?v=YGLNyHd2w10

Result of running the program against a moderately complex puzzle:
![graph renderer example](./_content/game_graph.gif)

## Setup

Install prerequisites:
- stable rust version (1.90.0 at time of writing)
- node 22 or greater

Run setup:
```bash
cd graph-view
```
```bash
cd RulesEngine
cargo test
```


The goal of the game is to push boxes around until every target is covered by a box.
Configure your map. currently written directly in `RulesEngine/src/main.rs` as a string. Defined as so:
```
#: wall
@: player
$: box
.: target
```

Once your map is configured, play it (optional)
```bash
cd RulesEngine
cargo run interactive
```

Then generate the state graph and view it in the native bevy application. Be sure to run in release mode, this easily 10x the runtime speed.
The generation can take a long time and produce a large graph for even moderately complex puzzles.
If the graph is too large, try to reduce the number of different valid states the puzzle can be in.
```bash
cd RulesEngine
cargo run --release graph
```