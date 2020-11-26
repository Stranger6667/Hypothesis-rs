use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intervalset;

fn create_new(c: &mut Criterion) {
    let value = black_box([(1, 1), (2, 3), (4, 6), (7, 9)]);
    c.bench_function("new", |b| {
        b.iter(|| {
            intervalset::IntervalSet::new(&value);
        })
    });
}

criterion_group!(default, create_new);
criterion_main!(default);
