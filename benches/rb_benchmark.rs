#![feature(random)]
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::random::random;

use blackbox::ringbuff::RingBuff;

fn rb_bench(c: &mut Criterion) {
    let numbers = black_box(
        (0..1 << 20)
            .map(|_| random::<i64>() as f32)
            .collect::<Vec<f32>>(),
    );
    let mut rb = RingBuff::<f32>::with_capacity(1 << 20);
    c.bench_function("rb write 1 << 20", |b| {
        b.iter(|| rb.push_slice(black_box(numbers.as_slice())))
    });
    // let mut rb = RingBuff::<f32, {1 << 20}>::new();
    // c.bench_function("rb write double 1 << 20", |b| b.iter(|| rb.push_slice_double(black_box(numbers.as_slice()))));
    // let mut rb = RingBuff::<f32, {1 << 20}>::new();
    // c.bench_function("rb write double closure 1 << 20", |b| b.iter(|| rb.push_slice_double_closure(black_box(numbers.as_slice()))));
    // let mut rb = RingBuff::<f32, {1 << 20}>::new();
    // c.bench_function("rb write for 1 << 20", |b| b.iter(|| rb.push_slice_for(black_box(numbers.as_slice()))));
    c.bench_function("rb read 1 << 20", |b| b.iter(|| rb.vectorize()));
}
criterion_group!(benches, rb_bench);
criterion_main!(benches);
