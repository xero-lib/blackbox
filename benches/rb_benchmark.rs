#![feature(random)]
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::random::random;

use blackbox::ringbuff::RingBuff;

fn rb_write(c: &mut Criterion) {
    let mut rb = RingBuff::<f32, { 1 << 20 }>::new();
    let numbers = (0..1 << 20)
        .map(|_| random::<i64>() as f32)
        .collect::<Vec<f32>>();

    c.bench_function("rb write 1 << 20", |b| {
        b.iter(|| rb.push_slice(black_box(numbers.as_slice())))
    });
}

fn rb_write_for(c: &mut Criterion) {
    let mut rb = RingBuff::<f32, { 1 << 20 }>::new();
    let numbers = (0..1 << 20)
        .map(|_| random::<i64>() as f32)
        .collect::<Vec<f32>>();

    c.bench_function("rb write for 1 << 20", |b| {
        b.iter(|| rb.push_slice_for(black_box(numbers.as_slice())))
    });
}

fn rb_read(c: &mut Criterion) {
    let mut rb = RingBuff::<f32, { 1 << 20 }>::new();
    let numbers = (0..1 << 20)
        .map(|_| random::<i64>() as f32)
        .collect::<Vec<f32>>();
    rb.push_slice(black_box(numbers.as_slice()));

    c.bench_function("rb read 1 << 20", |b| b.iter(|| rb.vectorize()));
}

criterion_group!(benches, rb_write, rb_write_for, rb_read);
criterion_main!(benches);
