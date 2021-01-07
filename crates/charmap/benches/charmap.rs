use charmap;
use charmap::{CategoryBitMap, UnicodeCategory};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_charmap(c: &mut Criterion) {
    let version = black_box(charmap::UnicodeVersion::V13);
    c.bench_function("create charmap", |b| {
        b.iter(|| {
            charmap::make_charmap(version);
        })
    });
    c.bench_function("create categories", |b| {
        b.iter(|| {
            charmap::make_categories(version);
        })
    });
    c.bench_function("as_general_categories", |b| {
        b.iter(|| {
            let _ = version.as_general_categories(&["N"]);
        })
    });
    let map = charmap::UnicodeVersion::V13.charmap();
    c.bench_function("get from charmap", |b| {
        b.iter(|| {
            let _ = map.get("Lu");
        })
    });
}

fn union_intervals(c: &mut Criterion) {
    let map = charmap::UnicodeVersion::V13.charmap();
    c.bench_function("union intervals", |b| {
        b.iter(|| {
            let lowercase = black_box(map.get("Ll").unwrap().to_vec());
            let uppercase = black_box(map.get("Lu").unwrap());
            charmap::union_intervals(lowercase, uppercase);
        })
    });
}

fn subtract_intervals(c: &mut Criterion) {
    let map = charmap::UnicodeVersion::V13.charmap();
    let uppercase = black_box(map.get("Lu").unwrap());
    c.bench_function("subtract intervals", |b| {
        b.iter(|| {
            let lowercase = black_box(map.get("Ll").unwrap().to_vec());
            charmap::subtract_intervals(lowercase, uppercase);
        })
    });
}

fn intervals(c: &mut Criterion) {
    let string = "abcdef0123456789";
    c.bench_function("char intervals", |b| {
        b.iter(|| {
            charmap::intervals(string);
        })
    });
}

fn category_key(c: &mut Criterion) {
    let version = black_box(charmap::UnicodeVersion::V13);
    let exclude = black_box(UnicodeCategory::So);
    let include = black_box(
        UnicodeCategory::Lu | UnicodeCategory::Me | UnicodeCategory::Cs | UnicodeCategory::Cc,
    );
    c.bench_function("category_key", |b| {
        b.iter(|| {
            charmap::category_key(version, exclude.into(), Some(include));
        })
    });
    c.bench_function("category_key_no_exclude", |b| {
        b.iter(|| {
            charmap::category_key(version, black_box(CategoryBitMap::new()), Some(include));
        })
    });
    c.bench_function("category_key_no_include", |b| {
        b.iter(|| {
            charmap::category_key(version, exclude.into(), black_box(None));
        })
    });
    c.bench_function("category_key_no_include_no_exclude", |b| {
        b.iter(|| {
            charmap::category_key(version, black_box(CategoryBitMap::new()), black_box(None));
        })
    });
}

fn query_for_key(c: &mut Criterion) {
    let version = black_box(charmap::UnicodeVersion::V13);
    let key = black_box(UnicodeCategory::Zl | UnicodeCategory::Zp | UnicodeCategory::Co);
    c.bench_function("query_for_key", |b| {
        b.iter(|| {
            charmap::query_for_key(version, key);
        })
    });
}

fn query(c: &mut Criterion) {
    let version = black_box(charmap::UnicodeVersion::V13);
    let exclude_categories = black_box(None);
    let include_categories = black_box(UnicodeCategory::Lu);
    let min_codepoint = black_box(Some(0));
    let max_codepoint = black_box(Some(128));
    let include_characters = black_box(Some("☃"));
    let exclude_characters = black_box(None);
    c.bench_function("query_top_level", |b| {
        b.iter(|| {
            let _ = version.query(
                exclude_categories,
                Some(include_categories),
                min_codepoint,
                max_codepoint,
                include_characters,
                exclude_characters,
            );
        })
    });
    c.bench_function("query_top_level_with_exclude", |b| {
        b.iter(|| {
            let _ = version.query(
                exclude_categories,
                Some(include_categories),
                min_codepoint,
                max_codepoint,
                include_characters,
                black_box(Some("A@т")),
            );
        })
    });
    c.bench_function("query_top_level_many_chars", |b| {
        b.iter(|| {
            let _ = version.query(
                exclude_categories,
                Some(include_categories),
                min_codepoint,
                max_codepoint,
                black_box(Some("0123456789")),
                black_box(Some("QWERTYUIOP")),
            );
        })
    });
}

criterion_group!(
    default,
    bench_charmap,
    union_intervals,
    subtract_intervals,
    intervals,
    category_key,
    query_for_key,
    query
);
criterion_main!(default);
