use blt_core::{run_tokenizer, CoreConfig};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;
use tokio::runtime::Runtime;

fn create_test_file(size_mb: usize) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_file.dat");
    let mut file = File::create(&file_path).unwrap();
    let chunk = vec![0u8; 1024 * 1024]; // 1MB chunk
    for _ in 0..size_mb {
        file.write_all(&chunk).unwrap();
    }
    (dir, file_path)
}

fn benchmark_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("Pipeline Benchmarks");

    // 100MB benchmark (existing)
    let (_in_dir_100, in_path_100) = create_test_file(100); // 100MB test file
    let out_dir_100 = tempdir().unwrap();
    let runtime_100 = Runtime::new().unwrap();
    group.sample_size(10);
    group.bench_function("passthrough_100mb_file", |b| {
        b.to_async(&runtime_100).iter(|| {
            let out_path = out_dir_100
                .path()
                .join(format!("output_{}.dat", rand::random::<u64>()));
            let config = CoreConfig::new_from_cli(
                Some(black_box(in_path_100.clone())),
                Some(black_box(out_path)),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            async {
                let result = run_tokenizer(config).await;
                result.unwrap();
                black_box(());
            }
        })
    });

    // 10MB benchmark
    let (_in_dir_10, in_path_10) = create_test_file(10); // 10MB test file
    let out_dir_10 = tempdir().unwrap();
    let runtime_10 = Runtime::new().unwrap();
    group.sample_size(10);
    group.bench_function("passthrough_10mb_file", |b| {
        b.to_async(&runtime_10).iter(|| {
            let out_path = out_dir_10
                .path()
                .join(format!("output_{}.dat", rand::random::<u64>()));
            let config = CoreConfig::new_from_cli(
                Some(black_box(in_path_10.clone())),
                Some(black_box(out_path)),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            async {
                let result = run_tokenizer(config).await;
                result.unwrap();
                black_box(());
            }
        })
    });

    // 1GB benchmark
    let (_in_dir_1g, in_path_1g) = create_test_file(1024); // 1GB test file
    let out_dir_1g = tempdir().unwrap();
    let runtime_1g = Runtime::new().unwrap();
    group.sample_size(10); // Criterion requires at least 10 samples
    group.bench_function("passthrough_1gb_file", |b| {
        b.to_async(&runtime_1g).iter(|| {
            let out_path = out_dir_1g
                .path()
                .join(format!("output_{}.dat", rand::random::<u64>()));
            let config = CoreConfig::new_from_cli(
                Some(black_box(in_path_1g.clone())),
                Some(black_box(out_path)),
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            async {
                let result = run_tokenizer(config).await;
                result.unwrap();
                black_box(());
            }
        })
    });

    group.finish();
}

criterion_group!(benches, benchmark_pipeline);
criterion_main!(benches);
