use criterion::{criterion_main};

mod octree_benchmarks;
use octree_benchmarks::*;
mod game_solve_benchmarks;
use game_solve_benchmarks::*;

criterion_main!(octree_benches, game_solve_benches);