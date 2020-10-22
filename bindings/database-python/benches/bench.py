import pytest

import database as rdb
import hypothesis.database as pydb


@pytest.fixture(params=[pydb, rdb])
def directory_db(request):
    tmp_dir = str(request.getfixturevalue("tmp_path"))
    return request.param.DirectoryBasedExampleDatabase(tmp_dir)


@pytest.fixture(params=[pydb, rdb])
def inmemory_db(request):
    return request.param.InMemoryExampleDatabase()


def _bench_save_one(benchmark, db):
    benchmark(db.save, b"foo", b"bar")


def _bench_save_multiple(benchmark, db):

    def save():
        for value in (b"bar1", b"bar2", b"bar3", b"bar4", b"bar5"):
            db.save(b"foo", value)

    benchmark(save)


def _bench_fetch_empty(benchmark, db):
    benchmark(lambda: list(db.fetch(b"foo")))


def _bench_fetch_one(benchmark, db):
    db.save(b"foo", b"bar")
    benchmark(lambda: list(db.fetch(b"foo")))


def _bench_fetch_multiple(benchmark, db):
    for value in (b"bar1", b"bar2", b"bar3", b"bar4", b"bar5"):
        db.save(b"foo", value)
    benchmark(lambda: list(db.fetch(b"foo")))


def _bench_delete_not_existing_key(benchmark, db):
    benchmark(db.delete, b"foo", b"bar")


def _bench_delete_existing_key_no_value(benchmark, db):
    db.save(b"foo", b"baz")
    benchmark(db.delete, b"foo", b"bar")


def _bench_delete_existing_key(benchmark, db):

    def bench():
        db.save(b"foo", b"bar")
        db.delete(b"foo", b"bar")

    benchmark(bench)


def _bench_move_the_same(benchmark, db):
    db.save(b"foo", b"bar")
    benchmark(db.move, b"foo", b"foo", b"bar")


def _bench_move_different(benchmark, db):
    db.save(b"foo", b"bar")
    benchmark(db.move, b"foo", b"baz", b"bar")


BENCHMARKS = {
    "Save one": _bench_save_one,
    "Save multiple": _bench_save_multiple,
    "Fetch empty": _bench_fetch_empty,
    "Fetch one": _bench_fetch_one,
    "Fetch multiple": _bench_fetch_multiple,
    "Delete not existing key": _bench_delete_not_existing_key,
    "Delete existing key no value": _bench_delete_existing_key_no_value,
    "Delete existing key": _bench_delete_existing_key,
    "Move same key": _bench_move_the_same,
    "Move different key": _bench_move_different,
}

for group, bench_func in BENCHMARKS.items():
    for storage in ("directory", "inmemory"):
        exec(f"""
@pytest.mark.benchmark(group="{storage.capitalize()}: {group}")
def test_{bench_func.__name__}_{storage}(benchmark, {storage}_db):
    {bench_func.__name__}(benchmark, {storage}_db)
        """)
