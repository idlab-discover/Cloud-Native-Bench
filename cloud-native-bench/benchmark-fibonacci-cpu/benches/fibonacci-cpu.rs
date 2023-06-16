use benchmark_fibonacci_cpu::fibonacci;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

pub fn cpu_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("criterion_benchmark");

    for num in [5, 6, 7, 8, 9, 10].iter() {
        group.bench_with_input(BenchmarkId::new("fibonacci", num), num, |b, &num| {
            b.iter(|| fibonacci(black_box(num)));
        });
    }

    group.finish();
}

criterion_group!(benches, cpu_benchmark);
criterion_main!(benches);
