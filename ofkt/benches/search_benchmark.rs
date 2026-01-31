use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ofkt::core::search::SearchEngine;
use ofkt::data::models::FileAlias;
use std::path::PathBuf;
use chrono::{Utc, Duration};

/// テスト用のエイリアスを作成
fn create_test_alias(alias: &str, path: &str) -> FileAlias {
    let now = Utc::now();
    FileAlias {
        id: uuid::Uuid::new_v4().to_string(),
        alias: alias.to_string(),
        path: PathBuf::from(path),
        tags: vec![],
        color: None,
        created_at: now,
        last_accessed: now - Duration::days(100),
        is_favorite: false,
    }
}

/// 大量のテストデータを生成
fn generate_test_data(count: usize) -> Vec<FileAlias> {
    let mut aliases = Vec::new();

    // 様々なパターンのエイリアスを生成
    for i in 0..count {
        let alias = match i % 5 {
            0 => format!("config_{}", i),
            1 => format!("document_{}", i),
            2 => format!("report_{}", i),
            3 => format!("試算表_{}", i),
            _ => format!("file_{}", i),
        };

        let path = match i % 3 {
            0 => format!("C:/Documents/{}/file.txt", i),
            1 => format!("C:/Projects/{}/code.rs", i),
            _ => format!("C:/Data/{}/data.xlsx", i),
        };

        let mut file_alias = create_test_alias(&alias, &path);

        // 一部のエイリアスにタグを追加
        if i % 4 == 0 {
            file_alias.tags = vec!["important".to_string(), "work".to_string()];
        }

        // 一部のエイリアスをお気に入りに設定
        if i % 7 == 0 {
            file_alias.is_favorite = true;
        }

        aliases.push(file_alias);
    }

    aliases
}

/// 完全一致検索のベンチマーク
fn bench_exact_match(c: &mut Criterion) {
    let aliases = generate_test_data(100);
    let mut engine = SearchEngine::with_aliases(aliases);

    c.bench_function("search_exact_match", |b| {
        b.iter(|| {
            let results = engine.search(black_box("config_10"));
            black_box(results);
        })
    });
}

/// 前方一致検索のベンチマーク
fn bench_prefix_match(c: &mut Criterion) {
    let aliases = generate_test_data(100);
    let mut engine = SearchEngine::with_aliases(aliases);

    c.bench_function("search_prefix_match", |b| {
        b.iter(|| {
            let results = engine.search(black_box("config_"));
            black_box(results);
        })
    });
}

/// ファジーマッチ検索のベンチマーク
fn bench_fuzzy_match(c: &mut Criterion) {
    let aliases = generate_test_data(100);
    let mut engine = SearchEngine::with_aliases(aliases);

    c.bench_function("search_fuzzy_match", |b| {
        b.iter(|| {
            let results = engine.search(black_box("cnfg"));
            black_box(results);
        })
    });
}

/// 階層パス検索のベンチマーク
fn bench_hierarchical_search(c: &mut Criterion) {
    let aliases = generate_test_data(100);
    let mut engine = SearchEngine::with_aliases(aliases);

    c.bench_function("search_hierarchical", |b| {
        b.iter(|| {
            let results = engine.search(black_box("Documents file"));
            black_box(results);
        })
    });
}

/// タグ検索のベンチマーク
fn bench_tag_search(c: &mut Criterion) {
    let aliases = generate_test_data(100);
    let mut engine = SearchEngine::with_aliases(aliases);

    c.bench_function("search_tag", |b| {
        b.iter(|| {
            let results = engine.search(black_box("important"));
            black_box(results);
        })
    });
}

/// データサイズによるスケーラビリティベンチマーク
fn bench_search_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_scalability");

    for size in [10, 50, 100, 500, 1000].iter() {
        let aliases = generate_test_data(*size);
        let mut engine = SearchEngine::with_aliases(aliases);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let results = engine.search(black_box("config"));
                black_box(results);
            });
        });
    }

    group.finish();
}

/// キャッシュ効果のベンチマーク
fn bench_cache_performance(c: &mut Criterion) {
    let aliases = generate_test_data(100);
    let mut engine = SearchEngine::with_aliases(aliases);

    // キャッシュなし（初回検索）
    c.bench_function("search_no_cache", |b| {
        b.iter(|| {
            engine.clear_cache();
            let results = engine.search(black_box("config"));
            black_box(results);
        })
    });

    // キャッシュあり（2回目以降）
    c.bench_function("search_with_cache", |b| {
        // キャッシュを準備
        engine.search("config");

        b.iter(|| {
            let results = engine.search(black_box("config"));
            black_box(results);
        })
    });
}

/// 複雑なクエリのベンチマーク
fn bench_complex_query(c: &mut Criterion) {
    let aliases = generate_test_data(200);
    let mut engine = SearchEngine::with_aliases(aliases);

    c.bench_function("search_complex_query", |b| {
        b.iter(|| {
            let results = engine.search(black_box("試算表 Documents"));
            black_box(results);
        })
    });
}

criterion_group!(
    benches,
    bench_exact_match,
    bench_prefix_match,
    bench_fuzzy_match,
    bench_hierarchical_search,
    bench_tag_search,
    bench_search_scalability,
    bench_cache_performance,
    bench_complex_query
);
criterion_main!(benches);
