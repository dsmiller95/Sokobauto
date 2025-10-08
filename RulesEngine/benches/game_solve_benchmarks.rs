use std::arch::x86_64::_mm512_getexp_round_pd;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode, Throughput};
use std::hint::black_box;
use std::time::Duration;
use RulesEngine::console_interface::parse_level;
use RulesEngine::state_graph::{StateGraph, UniqueNode, populate_step, get_all_adjacent_nodes, PopulateResult};

const PUZZLES: &[(&str, &str, usize, SamplingMode)] = &[
    ("puzzle_0", r#"
    ####
    #@$#
    ####
    "#, 100, SamplingMode::Auto),
    ("puzzle_1", r#"
    ######
    #@$ .#
    ######
    "#, 100, SamplingMode::Auto),
    ("puzzle_2", r#"
    ######
    #@$  #
    # $. #
    # .  #
    ######
    "#, 100, SamplingMode::Auto),
    ("puzzle_3", r#"
    ########
    # @$  .#
    # $  $ #
    # .# $ #
    #..#   #
    ########
    "#, 50, SamplingMode::Auto),
    ("puzzle_4", r#"
       ####
########  ##
#          ###
# @$$ ##   ..#
# $$   ##  ..#
#         ####
###########
"#, 10, SamplingMode::Flat),
];

pub fn bench_game_solve_full_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_solve_full_graph");

    for &(puzzle_name, puzzle, sample_size, sample_mode) in PUZZLES {
        group.sample_size(sample_size);
        group.sampling_mode(sample_mode);
        group.bench_with_input(
            BenchmarkId::new("complete_graph", puzzle_name),
            &puzzle,
            |b, &puzzle| {
                b.iter_with_setup(
                    || {
                        let (game_state, shared) = parse_level(puzzle);
                        let mut state_graph = StateGraph::new();
                        let min_reachable_position = shared
                            .reachable_positions(&game_state)
                            .into_iter()
                            .min()
                            .unwrap();
                        let first_node = UniqueNode {
                            environment: game_state.environment,
                            minimum_reachable_player_position: min_reachable_position,
                        };
                        state_graph.upsert_state(first_node);
                        (state_graph, shared)
                    },
                    |(mut state_graph, shared)| {
                        loop {
                            let result = populate_step(
                                black_box(&mut state_graph),
                                black_box(&shared)
                            );
                            if let PopulateResult::AllVisited = result {
                                break;
                            }
                        }
                        black_box(state_graph)
                    },
                );
            },
        );
    }
    group.finish();
}

pub fn bench_game_solve_single_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_solve_single_node");

    for &(puzzle_name, puzzle, sample_size, sample_mode) in PUZZLES {
        group.sample_size(sample_size);
        group.sampling_mode(sample_mode);
        group.bench_with_input(
            BenchmarkId::new("single_node_expansion", puzzle_name),
            &puzzle,
            |b, &puzzle| {
                b.iter_with_setup(
                    || {
                        let (game_state, shared) = parse_level(puzzle);
                        let min_reachable_position = shared
                            .reachable_positions(&game_state)
                            .into_iter().min().unwrap();
                        let unique_node = UniqueNode {
                            environment: game_state.environment,
                            minimum_reachable_player_position: min_reachable_position,
                        };
                        (unique_node, shared)
                    },
                    |(unique_node, shared)| {
                        let adjacent_nodes = get_all_adjacent_nodes(
                            black_box(&unique_node),
                            black_box(&shared)
                        );
                        black_box(adjacent_nodes)
                    },
                );
            },
        );
    }
    group.finish();
}

criterion_group!(
    game_solve_benches,
    bench_game_solve_full_graph, bench_game_solve_single_node
);

criterion_main!(game_solve_benches);