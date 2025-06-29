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

    // Create a large temporary file for input
    let (_in_dir, in_path) = create_test_file(100); // 100MB test file

    // Create a temporary directory for output files
    let out_dir = tempdir().unwrap();

    // Create a Tokio runtime for our async benchmarks
    let runtime = Runtime::new().unwrap();

    group.sample_size(10); // Run fewer samples because it's a long test
    group.bench_function("passthrough_100mb_file", |b| {
        // Use the runtime to run the async benchmark
        b.to_async(&runtime).iter(|| {
            // For each iteration, we need a unique output path
            let out_path = out_dir
                .path()
                .join(format!("output_{}.dat", rand::random::<u64>()));

            let config = CoreConfig::new_from_cli(
                Some(black_box(in_path.clone())),
                Some(black_box(out_path)),
                None, // No BPE merges -> PassthroughStrategy
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
