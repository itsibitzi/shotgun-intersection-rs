use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::prelude::*;
use shotgun_intersection::{galloping_intersect, intersect as shotgun_intersect};
use std::{cmp::Ordering, hint::black_box};

// Naive two-pointer intersection
fn naive_intersect<'a, T: Ord>(a: &'a [T], b: &'a [T]) -> Vec<&'a T> {
    let mut i = 0;
    let mut j = 0;
    let mut result = Vec::new();
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                result.push(&a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    result
}

// Generate sorted random data with some overlap
fn make_data(len_a: usize, len_b: usize, overlap: f64, seed: u64) -> (Vec<u32>, Vec<u32>) {
    let mut rng = StdRng::seed_from_u64(seed);
    let overlap_count = (len_a.min(len_b) as f64 * overlap) as usize;
    let mut shared: Vec<u32> = (0..overlap_count)
        .map(|_| rng.random_range(0..10_000_000))
        .collect();
    shared.sort_unstable();
    shared.dedup();

    let mut a: Vec<u32> = shared.clone();
    a.extend((0..(len_a - shared.len())).map(|_| rng.random_range(0..10_000_000)));
    a.sort_unstable();
    a.dedup();

    let mut b: Vec<u32> = shared;
    b.extend((0..(len_b - b.len())).map(|_| rng.random_range(0..10_000_000)));
    b.sort_unstable();
    b.dedup();

    (a, b)
}

fn bench_intersections(c: &mut Criterion) {
    let sizes = [
        (32, 100_000_000),
        // (10_000, 10_000),
        // (100_000, 100_000),
        // (1_000_000, 1_000_000),
        // (10_000, 1_000_000),
    ];
    let overlap = 0.1;
    let seed = 42;

    for &(len_a, len_b) in &sizes {
        let (a, b) = make_data(len_a, len_b, overlap, seed);

        let count1 = shotgun_intersect(&a, &b).count();
        let count2 = galloping_intersect(&a, &b).len();
        let count3 = naive_intersect(&a, &b).len();

        assert_eq!(count1, count2);
        assert_eq!(count2, count3);

        c.bench_with_input(
            BenchmarkId::new("Shotgun", format!("{len_a}x{len_b}")),
            &(a.as_slice(), b.as_slice()),
            |b, (a, b_)| {
                b.iter(|| {
                    let count = shotgun_intersect(black_box(a), black_box(b_)).count();
                    black_box(count)
                })
            },
        );

        c.bench_with_input(
            BenchmarkId::new("Galloping", format!("{len_a}x{len_b}")),
            &(a.as_slice(), b.as_slice()),
            |b, (a, b_)| {
                b.iter(|| {
                    let count = galloping_intersect(black_box(a), black_box(b_)).len();
                    black_box(count)
                })
            },
        );

        c.bench_with_input(
            BenchmarkId::new("Naive", format!("{len_a}x{len_b}")),
            &(a.as_slice(), b.as_slice()),
            |b, (a, b_)| {
                b.iter(|| {
                    let count = naive_intersect(black_box(a), black_box(b_)).len();
                    black_box(count)
                })
            },
        );
    }
}

criterion_group!(benches, bench_intersections);
criterion_main!(benches);
