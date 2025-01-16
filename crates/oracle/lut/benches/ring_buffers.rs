use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::VecDeque;

pub struct FixedRingBuffer<T, const N: usize> {
    ring_buffer: [T; N],
    current_index: usize,
}

impl<T, const N: usize> FixedRingBuffer<T, N> {
    pub const fn len(&self) -> usize {
        N
    }

    pub fn push_or_overwrite(&mut self, value: T) {
        let index = self.current_index;
        self.ring_buffer[index] = value;
        self.current_index = (index + 1) % N;
    }
}

impl<T: Copy + Default, const N: usize> Default for FixedRingBuffer<T, N> {
    fn default() -> Self {
        FixedRingBuffer {
            ring_buffer: [T::default(); N],
            current_index: 0,
        }
    }
}

fn bench_ring_buffer(c: &mut Criterion) {
    c.bench_function("FixedRingBuffer", |b| {
        let mut ring_buffer = FixedRingBuffer::<usize, 10000>::default();
        b.iter(|| {
            for i in 0..100000 {
                black_box(ring_buffer.push_or_overwrite(i));
            }
        });
    });
}

fn bench_vecdeque(c: &mut Criterion) {
    c.bench_function("VecDeque", |b| {
        let mut ring_buffer = VecDeque::<usize>::with_capacity(10000);
        b.iter(|| {
            for i in 0..100000 {
                if ring_buffer.capacity() == ring_buffer.len() {
                    black_box(ring_buffer.pop_back());
                }
                black_box(ring_buffer.push_back(i));
            }
        });
    });
}

criterion_group!(benches, bench_ring_buffer, bench_vecdeque);
criterion_main!(benches);
