use criterion::{black_box, criterion_group, criterion_main, Criterion};
use database::{DirectoryBasedExampleDatabase, ExampleDatabase, InMemoryExampleDatabase};

const DATABASE_PATH: &'static str = "/tmp/.hypothesis-db-rs-test";

fn bench_save(c: &mut Criterion, id: &str, mut db: impl ExampleDatabase) {
    let key = black_box(b"foo");
    let value = black_box(b"bar");
    c.bench_function(id, |b| b.iter(|| db.save(key, value)));
}

fn bench_fetch_some(c: &mut Criterion, id: &str, mut db: impl ExampleDatabase) {
    db.save(b"foo", b"bar1");
    db.save(b"foo", b"bar2");
    let key = black_box(b"foo");
    c.bench_function(id, |b| b.iter(|| db.fetch(key).into_vec()));
}

fn bench_fetch_empty(c: &mut Criterion, id: &str, db: impl ExampleDatabase) {
    let key = black_box(b"unknown");
    c.bench_function(id, |b| b.iter(|| db.fetch(key).into_vec()));
}

fn bench_delete_not_existing_key(c: &mut Criterion, id: &str, mut db: impl ExampleDatabase) {
    let key = black_box(b"doesntexist");
    let value = black_box(b"doesntexist");
    c.bench_function(id, |b| b.iter(|| db.delete(key, value)));
}

fn bench_delete_existing_key_no_value(c: &mut Criterion, id: &str, mut db: impl ExampleDatabase) {
    db.save(b"foo", b"bar");
    let key = black_box(b"foo");
    let value = black_box(b"doesntexist");
    c.bench_function(id, |b| b.iter(|| db.delete(key, value)));
}

fn bench_delete_existing_key_value(c: &mut Criterion, id: &str, mut db: impl ExampleDatabase) {
    let key = black_box(b"foo");
    let value = black_box(b"bar");
    c.bench_function(id, |b| {
        b.iter(|| {
            db.save(b"foo", b"bar");
            db.delete(key, value)
        })
    });
}

fn directory_save(c: &mut Criterion) {
    let db = DirectoryBasedExampleDatabase::new(DATABASE_PATH);
    bench_save(c, "directory save", db);
}

fn directory_fetch_some(c: &mut Criterion) {
    let db = DirectoryBasedExampleDatabase::new(DATABASE_PATH);
    bench_fetch_some(c, "directory fetch some", db);
}

fn directory_fetch_empty(c: &mut Criterion) {
    let db = DirectoryBasedExampleDatabase::new(DATABASE_PATH);
    bench_fetch_empty(c, "directory fetch empty", db);
}

fn directory_delete_not_existing_key(c: &mut Criterion) {
    let db = DirectoryBasedExampleDatabase::new(DATABASE_PATH);
    bench_delete_not_existing_key(c, "directory delete not existing key", db);
}

fn directory_delete_existing_key_no_value(c: &mut Criterion) {
    let db = DirectoryBasedExampleDatabase::new(DATABASE_PATH);
    bench_delete_existing_key_no_value(c, "directory delete existing key no value", db);
}

fn directory_delete_existing_key_value(c: &mut Criterion) {
    let db = DirectoryBasedExampleDatabase::new(DATABASE_PATH);
    bench_delete_existing_key_value(c, "directory delete existing key value", db);
}

fn inmemory_save(c: &mut Criterion) {
    let db = InMemoryExampleDatabase::new();
    bench_save(c, "inmemory save", db);
}

fn inmemory_fetch_some(c: &mut Criterion) {
    let db = InMemoryExampleDatabase::new();
    bench_fetch_some(c, "inmemory fetch some", db);
}

fn inmemory_fetch_empty(c: &mut Criterion) {
    let db = InMemoryExampleDatabase::new();
    bench_fetch_empty(c, "inmemory fetch empty", db);
}

fn inmemory_delete_not_existing_key(c: &mut Criterion) {
    let db = InMemoryExampleDatabase::new();
    bench_delete_not_existing_key(c, "inmemory delete not existing key", db);
}

fn inmemory_delete_existing_key_no_value(c: &mut Criterion) {
    let db = InMemoryExampleDatabase::new();
    bench_delete_existing_key_no_value(c, "inmemory delete existing key no value", db);
}

fn inmemory_delete_existing_key_value(c: &mut Criterion) {
    let db = InMemoryExampleDatabase::new();
    bench_delete_existing_key_value(c, "inmemory delete existing key value", db);
}

criterion_group!(
    directory,
    directory_save,
    directory_fetch_some,
    directory_fetch_empty,
    directory_delete_not_existing_key,
    directory_delete_existing_key_no_value,
    directory_delete_existing_key_value,
);
criterion_group!(
    inmemory,
    inmemory_save,
    inmemory_fetch_some,
    inmemory_fetch_empty,
    inmemory_delete_not_existing_key,
    inmemory_delete_existing_key_no_value,
    inmemory_delete_existing_key_value
);
criterion_main!(directory, inmemory);
