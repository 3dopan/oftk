use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ofkt::core::file_manager::FileManager;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// テスト用のファイルを作成
fn create_test_file(dir: &TempDir, name: &str, size_kb: usize) -> PathBuf {
    let path = dir.path().join(name);
    let mut file = File::create(&path).unwrap();

    // 指定サイズのデータを書き込む
    let data = vec![b'X'; size_kb * 1024];
    file.write_all(&data).unwrap();

    path
}

/// テスト用のディレクトリ構造を作成
fn create_test_directory(dir: &TempDir, name: &str, file_count: usize) -> PathBuf {
    let path = dir.path().join(name);
    fs::create_dir(&path).unwrap();

    for i in 0..file_count {
        let file_path = path.join(format!("file_{}.txt", i));
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test data {}", i).unwrap();
    }

    path
}

/// ファイルコピーのベンチマーク（小さいファイル）
fn bench_copy_small_file(c: &mut Criterion) {
    c.bench_function("file_copy_1kb", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let src = create_test_file(&temp_dir, "source.txt", 1);
                let dest = temp_dir.path().join("dest.txt");
                (manager, src, dest, temp_dir)
            },
            |(manager, src, dest, _temp_dir)| {
                let _ = manager.copy(black_box(&src), black_box(&dest));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// ファイルコピーのベンチマーク（中サイズファイル）
fn bench_copy_medium_file(c: &mut Criterion) {
    c.bench_function("file_copy_100kb", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let src = create_test_file(&temp_dir, "source.txt", 100);
                let dest = temp_dir.path().join("dest.txt");
                (manager, src, dest, temp_dir)
            },
            |(manager, src, dest, _temp_dir)| {
                let _ = manager.copy(black_box(&src), black_box(&dest));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// ファイルコピーのベンチマーク（大きいファイル）
fn bench_copy_large_file(c: &mut Criterion) {
    c.bench_function("file_copy_1mb", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let src = create_test_file(&temp_dir, "source.txt", 1024);
                let dest = temp_dir.path().join("dest.txt");
                (manager, src, dest, temp_dir)
            },
            |(manager, src, dest, _temp_dir)| {
                let _ = manager.copy(black_box(&src), black_box(&dest));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// ファイルサイズによるコピーのスケーラビリティ
fn bench_copy_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_copy_scalability");

    for size_kb in [1, 10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}kb", size_kb)),
            size_kb,
            |b, &size_kb| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let manager = FileManager::new();
                        let src = create_test_file(&temp_dir, "source.txt", size_kb);
                        let dest = temp_dir.path().join("dest.txt");
                        (manager, src, dest, temp_dir)
                    },
                    |(manager, src, dest, _temp_dir)| {
                        let _ = manager.copy(black_box(&src), black_box(&dest));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

/// ファイル移動のベンチマーク
fn bench_move_file(c: &mut Criterion) {
    c.bench_function("file_move", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let src = create_test_file(&temp_dir, "source.txt", 10);
                let dest = temp_dir.path().join("dest.txt");
                (manager, src, dest, temp_dir)
            },
            |(manager, src, dest, _temp_dir)| {
                let _ = manager.move_file(black_box(&src), black_box(&dest));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// ファイル削除のベンチマーク（完全削除）
fn bench_delete_permanent(c: &mut Criterion) {
    c.bench_function("file_delete_permanent", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let file = create_test_file(&temp_dir, "to_delete.txt", 10);
                (manager, file, temp_dir)
            },
            |(manager, file, _temp_dir)| {
                let _ = manager.delete(black_box(&file), black_box(true));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// ディレクトリ削除のベンチマーク
fn bench_delete_directory(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_delete");

    for file_count in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_files", file_count)),
            file_count,
            |b, &file_count| {
                b.iter_batched(
                    || {
                        let temp_dir = TempDir::new().unwrap();
                        let manager = FileManager::new();
                        let dir = create_test_directory(&temp_dir, "to_delete", file_count);
                        (manager, dir, temp_dir)
                    },
                    |(manager, dir, _temp_dir)| {
                        let _ = manager.delete(black_box(&dir), black_box(true));
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

/// ファイル名変更のベンチマーク
fn bench_rename(c: &mut Criterion) {
    c.bench_function("file_rename", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let file = create_test_file(&temp_dir, "old_name.txt", 10);
                (manager, file, temp_dir)
            },
            |(manager, file, _temp_dir)| {
                let _ = manager.rename(black_box(&file), black_box("new_name.txt"));
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// 複数のファイル操作の組み合わせ
fn bench_multiple_operations(c: &mut Criterion) {
    c.bench_function("file_multiple_operations", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();

                // 複数のテストファイルを作成
                let file1 = create_test_file(&temp_dir, "file1.txt", 10);
                let file2 = create_test_file(&temp_dir, "file2.txt", 10);
                let file3 = create_test_file(&temp_dir, "file3.txt", 10);

                (manager, file1, file2, file3, temp_dir)
            },
            |(manager, file1, file2, file3, temp_dir)| {
                // コピー
                let copy_dest = temp_dir.path().join("copy.txt");
                let _ = manager.copy(&file1, &copy_dest);

                // 移動
                let move_dest = temp_dir.path().join("moved.txt");
                let _ = manager.move_file(&file2, &move_dest);

                // リネーム
                let _ = manager.rename(&file3, "renamed.txt");

                // 削除
                let _ = manager.delete(&copy_dest, true);

                black_box(&manager);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// エラーハンドリングのベンチマーク
fn bench_error_handling(c: &mut Criterion) {
    c.bench_function("file_error_handling", |b| {
        b.iter_batched(
            || {
                let temp_dir = TempDir::new().unwrap();
                let manager = FileManager::new();
                let nonexistent = temp_dir.path().join("nonexistent.txt");
                (manager, nonexistent, temp_dir)
            },
            |(manager, nonexistent, temp_dir)| {
                // 存在しないファイルに対する操作（エラーが返される）
                let dest = temp_dir.path().join("dest.txt");
                let _ = manager.copy(black_box(&nonexistent), black_box(&dest));
                let _ = manager.move_file(black_box(&nonexistent), black_box(&dest));
                let _ = manager.delete(black_box(&nonexistent), black_box(true));
                let _ = manager.rename(black_box(&nonexistent), black_box("new.txt"));
                black_box(&manager);
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    bench_copy_small_file,
    bench_copy_medium_file,
    bench_copy_large_file,
    bench_copy_scalability,
    bench_move_file,
    bench_delete_permanent,
    bench_delete_directory,
    bench_rename,
    bench_multiple_operations,
    bench_error_handling
);
criterion_main!(benches);
