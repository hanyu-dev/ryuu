use std::hint::black_box;
use std::{f32, f64};

use criterion::{criterion_group, criterion_main, Criterion};

// Test values for benchmarking
const F64_VALUES: &[f64] = &[
    0.0,
    0.1234,
    core::f64::consts::E,
    core::f64::consts::PI,
    1.23e40,
    1.23e-40,
    f64::MAX,
    f64::MIN,
    123.456_789,
    -123.456_789,
];

const F32_VALUES: &[f32] = &[
    0.0,
    0.1234,
    core::f32::consts::E,
    core::f32::consts::PI,
    1.23e20,
    1.23e-20,
    f32::MAX,
    f32::MIN,
    123.456_79,
    -123.456_79,
];

fn bench_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("f64");

    for &value in F64_VALUES {
        group.bench_function(format!("ryu/{value}"), |b| {
            b.iter(|| {
                let mut buf = ryu::Buffer::new();
                let string = buf.format(black_box(value));
                black_box(string);
            });
        });

        group.bench_function(format!("ryuu/{value}"), |b| {
            b.iter(|| {
                let formatted = ryuu::Formatter::format_f64(black_box(value));
                let string = formatted.as_str();
                black_box(string);
            });
        });

        group.bench_function(format!("std/{value}"), |b| {
            b.iter(|| format!("{value}"));
        });
    }

    group.finish();
}

fn bench_f32(c: &mut Criterion) {
    let mut group = c.benchmark_group("f32");

    for &value in F32_VALUES {
        group.bench_function(format!("ryu/{value}"), |b| {
            b.iter(|| {
                let mut buf = ryu::Buffer::new();
                let string = buf.format(black_box(value));
                black_box(string);
            });
        });

        group.bench_function(format!("ryuu/{value}"), |b| {
            b.iter(|| {
                let formatted = ryuu::Formatter::format_f32(black_box(value));
                let string = formatted.as_str();
                black_box(string);
            });
        });

        group.bench_function(format!("std/{value}"), |b| {
            b.iter(|| format!("{value}"));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_f64, bench_f32);
criterion_main!(benches);
