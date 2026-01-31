use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ofkt::core::alias::AliasManager;
use std::path::PathBuf;

/// エイリアス追加のベンチマーク
fn bench_add_alias(c: &mut Criterion) {
    c.bench_function("alias_add_single", |b| {
        b.iter(|| {
            let mut manager = AliasManager::new();
            manager.add_alias(
                black_box("test_alias".to_string()),
                black_box(PathBuf::from("/path/to/file")),
                black_box(vec![]),
                black_box(None),
                black_box(false),
            )
        })
    });
}

/// 複数エイリアス追加のベンチマーク
fn bench_add_multiple_aliases(c: &mut Criterion) {
    let mut group = c.benchmark_group("alias_add_multiple");

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut manager = AliasManager::new();
                for i in 0..count {
                    let _ = manager.add_alias(
                        format!("alias_{}", i),
                        PathBuf::from(format!("/path/to/file_{}", i)),
                        vec![],
                        None,
                        false,
                    );
                }
                black_box(manager);
            })
        });
    }

    group.finish();
}

/// タグ付きエイリアス追加のベンチマーク
fn bench_add_alias_with_tags(c: &mut Criterion) {
    c.bench_function("alias_add_with_tags", |b| {
        b.iter(|| {
            let mut manager = AliasManager::new();
            manager.add_alias(
                black_box("tagged_alias".to_string()),
                black_box(PathBuf::from("/path/to/file")),
                black_box(vec!["important".to_string(), "work".to_string(), "project".to_string()]),
                black_box(Some("#FF0000".to_string())),
                black_box(true),
            )
        })
    });
}

/// エイリアス削除のベンチマーク
fn bench_remove_alias(c: &mut Criterion) {
    c.bench_function("alias_remove_by_id", |b| {
        b.iter_batched(
            || {
                // セットアップ: エイリアスを追加
                let mut manager = AliasManager::new();
                for i in 0..100 {
                    let _ = manager.add_alias(
                        format!("alias_{}", i),
                        PathBuf::from(format!("/path/to/file_{}", i)),
                        vec![],
                        None,
                        false,
                    );
                }
                let id = manager.get_aliases()[50].id.clone();
                (manager, id)
            },
            |(mut manager, id)| {
                let _ = manager.remove_alias_by_id(&id);
                black_box(manager);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("alias_remove_by_name", |b| {
        b.iter_batched(
            || {
                // セットアップ: エイリアスを追加
                let mut manager = AliasManager::new();
                for i in 0..100 {
                    let _ = manager.add_alias(
                        format!("alias_{}", i),
                        PathBuf::from(format!("/path/to/file_{}", i)),
                        vec![],
                        None,
                        false,
                    );
                }
                manager
            },
            |mut manager| {
                let _ = manager.remove_alias_by_name("alias_50");
                black_box(manager);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// エイリアス更新のベンチマーク
fn bench_update_alias(c: &mut Criterion) {
    c.bench_function("alias_update_single_field", |b| {
        b.iter_batched(
            || {
                // セットアップ: エイリアスを追加
                let mut manager = AliasManager::new();
                let _ = manager.add_alias(
                    "test".to_string(),
                    PathBuf::from("/path/to/file"),
                    vec![],
                    None,
                    false,
                );
                let id = manager.get_aliases()[0].id.clone();
                (manager, id)
            },
            |(mut manager, id)| {
                let _ = manager.update_alias(
                    &id,
                    Some(black_box("updated".to_string())),
                    None,
                    None,
                    None,
                    None,
                );
                black_box(manager);
            },
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("alias_update_all_fields", |b| {
        b.iter_batched(
            || {
                // セットアップ: エイリアスを追加
                let mut manager = AliasManager::new();
                let _ = manager.add_alias(
                    "test".to_string(),
                    PathBuf::from("/path/to/file"),
                    vec![],
                    None,
                    false,
                );
                let id = manager.get_aliases()[0].id.clone();
                (manager, id)
            },
            |(mut manager, id)| {
                let _ = manager.update_alias(
                    &id,
                    Some(black_box("updated".to_string())),
                    Some(black_box(PathBuf::from("/new/path"))),
                    Some(black_box(vec!["tag1".to_string(), "tag2".to_string()])),
                    Some(black_box(Some("#00FF00".to_string()))),
                    Some(black_box(true)),
                );
                black_box(manager);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// エイリアス取得のベンチマーク
fn bench_get_aliases(c: &mut Criterion) {
    let mut group = c.benchmark_group("alias_get_all");

    for count in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            // セットアップ: 指定数のエイリアスを追加
            let mut manager = AliasManager::new();
            for i in 0..count {
                let _ = manager.add_alias(
                    format!("alias_{}", i),
                    PathBuf::from(format!("/path/to/file_{}", i)),
                    vec![],
                    None,
                    false,
                );
            }

            b.iter(|| {
                let aliases = manager.get_aliases();
                black_box(aliases);
            })
        });
    }

    group.finish();
}

/// 重複チェックのベンチマーク
fn bench_duplicate_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("alias_duplicate_check");

    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_batched(
                || {
                    // セットアップ: 指定数のエイリアスを追加
                    let mut manager = AliasManager::new();
                    for i in 0..count {
                        let _ = manager.add_alias(
                            format!("alias_{}", i),
                            PathBuf::from(format!("/path/to/file_{}", i)),
                            vec![],
                            None,
                            false,
                        );
                    }
                    manager
                },
                |mut manager| {
                    // 既存のエイリアスと同じ名前で追加を試みる
                    let result = manager.add_alias(
                        black_box("alias_50".to_string()),
                        PathBuf::from("/duplicate/path"),
                        vec![],
                        None,
                        false,
                    );
                    black_box(result);
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// 複雑な操作のベンチマーク（追加・更新・削除の組み合わせ）
fn bench_complex_operations(c: &mut Criterion) {
    c.bench_function("alias_complex_operations", |b| {
        b.iter(|| {
            let mut manager = AliasManager::new();

            // 10個追加
            for i in 0..10 {
                let _ = manager.add_alias(
                    format!("alias_{}", i),
                    PathBuf::from(format!("/path/to/file_{}", i)),
                    vec![format!("tag_{}", i)],
                    None,
                    i % 3 == 0,
                );
            }

            // 5個更新
            for i in 0..5 {
                let id = manager.get_aliases()[i].id.clone();
                let _ = manager.update_alias(
                    &id,
                    Some(format!("updated_{}", i)),
                    None,
                    None,
                    None,
                    Some(true),
                );
            }

            // 3個削除
            for _ in 0..3 {
                if !manager.get_aliases().is_empty() {
                    let id = manager.get_aliases()[0].id.clone();
                    let _ = manager.remove_alias_by_id(&id);
                }
            }

            black_box(manager);
        })
    });
}

criterion_group!(
    benches,
    bench_add_alias,
    bench_add_multiple_aliases,
    bench_add_alias_with_tags,
    bench_remove_alias,
    bench_update_alias,
    bench_get_aliases,
    bench_duplicate_check,
    bench_complex_operations
);
criterion_main!(benches);
