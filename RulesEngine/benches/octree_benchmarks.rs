use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box;
use RulesEngine::bevy_interface::octree::{Octree};

use bevy::prelude::Vec3;
use rand::prelude::*;
use rand::SeedableRng;

fn generate_random_points(count: usize, bounds_size: f32, rng: &mut impl RngCore) -> Vec<(usize, Vec3)> {
    let mut points = Vec::new();
    for i in 0..count {
        let pos = bounds_size * rng.random::<Vec3>();
        points.push((i, pos));
    }
    points
}

fn bench_octree_creation(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(12345);
    let mut group = c.benchmark_group("octree_creation");

    const BOUNDS_SIZE: f32 = 100.0;
    for &size in &[100, 500, 1000, 2000, 5000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("from_points", size),
            &size,
            |b, size| {
                b.iter_with_setup(
                    || {
                        let points = generate_random_points(*size, BOUNDS_SIZE, &mut rng);
                        points
                    },
                    |points| {
                    black_box(Octree::from_points(
                        black_box(&points),
                        black_box(8),
                        black_box(10),
                    ))
                });
            },
        );
    }
    group.finish();
}

fn bench_octree_movement(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(12345);
    let mut group = c.benchmark_group("octree_movement");
    
    const BOUNDS_SIZE: f32 = 100.0;
    const MAX_MOVE_DISTANCE: f32 = 5.0;

    let node_counts = [100, 500, 1000, 2000, 4000];

    for &node_count in &node_counts {
        group.throughput(Throughput::Elements(node_count as u64));
        group.bench_with_input(
            BenchmarkId::new("move_random_directions", node_count),
            &node_count,
            |b, &node_count| {
                b.iter_with_setup(
                    || {
                        let mut node_positions: HashMap<usize, Vec3> = HashMap::new();
                        let initial_points = generate_random_points(node_count, BOUNDS_SIZE, &mut rng);
                        for &(id, pos) in &initial_points {
                            node_positions.insert(id, pos);
                        }
                        let octree = Octree::from_points(&initial_points, 8, 10);
                        (octree, node_positions)
                    },
                    |(mut octree, mut node_positions)| {
                        let rng_state = 54321u64;
                        let mut rng = SmallRng::seed_from_u64(rng_state);

                        for node_id in 0..node_count {
                            let current_pos = node_positions.entry(node_id).or_insert(Vec3::splat(BOUNDS_SIZE/2.0));

                            let move_vec = (rng.random::<Vec3>() - Vec3::splat(-0.5)) * MAX_MOVE_DISTANCE * 2.0;
                            let mut new_pos = *current_pos + move_vec;
                            new_pos = new_pos.clamp(Vec3::ZERO, Vec3::splat(BOUNDS_SIZE));

                            octree.update(
                                black_box(node_id),
                                black_box(*current_pos),
                                black_box(new_pos)
                            );

                            *current_pos = new_pos;
                        }
                        black_box(octree)
                    },
                );
            },
        );
    }
    group.finish();
}

fn bench_octree_force_calculation(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(12345);
    let mut group = c.benchmark_group("octree_force_calculation");

    const BOUNDS_SIZE: f32 = 100.0;
    const NUM_TEST_POSITIONS: usize = 100;

    fn sample_force((octree, test_positions): (Octree, Vec<Vec3>)) -> Vec3 {
        let mut total_force = Vec3::ZERO;
        for &test_pos in test_positions.iter() {
            let force = octree.calculate_force(
                black_box(test_pos),
                black_box(0.5),
                black_box(1.0),
            );
            total_force += force;
        }
        black_box(total_force)
    }

    for &node_count in &[100, 500, 1000, 2000] {
        group.throughput(Throughput::Elements(node_count as u64));
        group.bench_with_input(
            BenchmarkId::new("barnes_hut", node_count),
            &node_count,
            |b, _| {
                b.iter_with_setup(
                    || {
                        let points = generate_random_points(node_count, BOUNDS_SIZE, &mut rng);
                        let octree = Octree::from_points(&points, 8, 10);
                        let test_positions = generate_random_points(NUM_TEST_POSITIONS, BOUNDS_SIZE, &mut rng)
                            .into_iter()
                            .map(|(_, pos)| pos)
                            .collect::<Vec<_>>();
                        (octree, test_positions)
                    },
                    sample_force);
            },
        );
        group.throughput(Throughput::Elements(node_count as u64));
        group.bench_with_input(
            BenchmarkId::new("after_moved_octree", node_count),
            &node_count,
            |b, _| {
                b.iter_with_setup(
                    || {
                        // let checker_size = BOUNDS_SIZE / 10.0;
                        let points = generate_random_points(node_count, BOUNDS_SIZE, &mut rng);
                        let moved_points = generate_random_points(node_count, BOUNDS_SIZE, &mut rng).into_iter().map(|(id, pos)| {
                            let should_keep = (pos.x % 5.0 < 0.1) && (pos.y % 5.0 < 0.1) && (pos.z % 5.0 < 0.1);
                            if should_keep {
                                (id, pos)
                            } else {
                                // round to super-dense cell at 1,1,1
                                (id, pos / BOUNDS_SIZE)
                            }
                        }).collect::<Vec<_>>();
                        let mut octree = Octree::from_points(&points, 8, 10);
                        for (id, new_pos) in moved_points {
                            let old_pos = points[id].1;
                            octree.update(id, old_pos, new_pos);
                        }
                        let test_positions = generate_random_points(NUM_TEST_POSITIONS, BOUNDS_SIZE, &mut rng)
                            .into_iter()
                            .map(|(_, pos)| pos)
                            .collect::<Vec<_>>();
                        (octree, test_positions)
                    },
                    sample_force);
            },
        );
    }
    group.finish();
}

fn bench_octree_different_theta(c: &mut Criterion) {
    let mut rng = SmallRng::seed_from_u64(12345);
    let mut group = c.benchmark_group("octree_theta_comparison");

    const NUM_NODES: usize = 1000;
    const NUM_TEST_POSITIONS: usize = 100;
    const BOUNDS_SIZE: f32 = 100.0;

    for &theta in &[0.1, 0.3, 0.4, 0.5, 0.6, 1.0, 2.0] {
        group.throughput(Throughput::Elements(NUM_TEST_POSITIONS as u64));
        group.bench_with_input(
            BenchmarkId::new("theta", theta),
            &theta,
            |b, &theta| {
                b.iter_with_setup(
                    || {
                        let points = generate_random_points(NUM_NODES, BOUNDS_SIZE, &mut rng);
                        let octree = Octree::from_points(&points, 8, 10);
                        let test_positions = generate_random_points(NUM_TEST_POSITIONS, BOUNDS_SIZE, &mut rng)
                            .into_iter()
                            .map(|(_, pos)| pos)
                            .collect::<Vec<_>>();
                        (octree, test_positions)
                    },
                    |(octree, test_positions)| {
                    let mut total_force = Vec3::ZERO;
                    for &test_pos in &test_positions {
                        let force = octree.calculate_force(
                            black_box(test_pos),
                            black_box(theta),
                            black_box(1.0),
                        );
                        total_force += force;
                    }
                    black_box(total_force)
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    octree_benches,
    bench_octree_creation,
    bench_octree_movement,
    bench_octree_force_calculation,
    bench_octree_different_theta,
);

criterion_main!(octree_benches);