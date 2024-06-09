use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use std::path::PathBuf;
use tempfile::TempDir;

use dev_hash::Crc32;
use fs_index::index::ResourceIndex;

// The path to the test assets directory
const DIR_PATH: &str = "../test-assets/";

fn resource_index_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_index");
    group.measurement_time(std::time::Duration::from_secs(20)); // Set the measurement time here

    let benchmarks_dir = setup_temp_dir();
    let benchmarks_dir = benchmarks_dir.path();
    let benchmarks_dir_str = benchmarks_dir.to_str().unwrap();

    // Benchmark `ResourceIndex::build()`

    let mut collisions_size = 0;
    group.bench_with_input(
        BenchmarkId::new("index_build", benchmarks_dir_str),
        &benchmarks_dir,
        |b, path| {
            b.iter(|| {
                let index: ResourceIndex<Crc32> =
                    ResourceIndex::build(black_box(path));
                collisions_size = index.collisions.len();
            });
        },
    );
    println!("Collisions: {}", collisions_size);

    // TODO: Benchmark `ResourceIndex::get_resource_by_id()`

    // TODO: Benchmark `ResourceIndex::get_resource_by_path()`

    // TODO: Benchmark `ResourceIndex::track_addition()`

    // TODO: Benchmark `ResourceIndex::track_deletion()`

    // TODO: Benchmark `ResourceIndex::track_update()`

    // Benchmark `ResourceIndex::update_all()`

    // First, create a new temp directory specifically for the update_all benchmark
    // since we will be creating new files, removing files, and modifying files
    let update_all_benchmarks_dir =
        TempDir::with_prefix("ark-fs-index-benchmarks-update-all").unwrap();
    let update_all_benchmarks_dir = update_all_benchmarks_dir.path();

    group.bench_function("index_update_all", |b| {
        b.iter(|| {
            // Clear the directory
            std::fs::remove_dir_all(&update_all_benchmarks_dir).unwrap();
            std::fs::create_dir(&update_all_benchmarks_dir).unwrap();

            // Create 50 new files
            for i in 0..50 {
                let new_file =
                    update_all_benchmarks_dir.join(format!("file_{}.txt", i));
                std::fs::File::create(&new_file).unwrap();
                std::fs::write(&new_file, format!("Hello, World! {}", i))
                    .unwrap();
            }
            let mut index: ResourceIndex<Crc32> =
                ResourceIndex::build(black_box(&update_all_benchmarks_dir));

            update_all_files(&update_all_benchmarks_dir.to_path_buf());
            let _update_result = index.update_all().unwrap();
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = resource_index_benchmark
}
criterion_main!(benches);

/// A helper function to setup a temp directory for the benchmarks using the test assets directory
fn setup_temp_dir() -> TempDir {
    // assert the path exists and is a directory
    assert!(
        std::path::Path::new(DIR_PATH).is_dir(),
        "The path: {} does not exist or is not a directory",
        DIR_PATH
    );

    // Create a temp directory
    let temp_dir = TempDir::with_prefix("ark-fs-index-benchmarks").unwrap();
    let benchmarks_dir = temp_dir.path();
    let benchmarks_dir_str = benchmarks_dir.to_str().unwrap();
    log::info!("Temp directory for benchmarks: {}", benchmarks_dir_str);

    // Copy the test assets to the temp directory
    let source = std::path::Path::new(DIR_PATH);
    // Can't use fs::copy because the source is a directory
    let output = std::process::Command::new("cp")
        .arg("-r")
        .arg(source)
        .arg(benchmarks_dir_str)
        .output()
        .expect("Failed to copy test assets to temp directory");
    if !output.status.success() {
        panic!(
            "Failed to copy test assets to temp directory: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    temp_dir
}

/// A helper function that takes a directory and creates 50 new files, removes 30 files, and modifies 10 files
///
/// Note: The function assumes that the directory already contains 50 files named `file_0.txt` to `file_49.txt`
fn update_all_files(dir: &PathBuf) {
    // Create 50 new files
    for i in 51..101 {
        let new_file = dir.join(format!("file_{}.txt", i));
        std::fs::File::create(&new_file).unwrap();
        // We add the index `i` to the file content to make sure the content is unique
        // This is to avoid collisions in the index
        std::fs::write(&new_file, format!("Hello, World! {}", i)).unwrap();
    }

    // Remove 30 files
    for i in 0..30 {
        let removed_file = dir.join(format!("file_{}.txt", i));
        std::fs::remove_file(&removed_file).unwrap();
    }

    // Modify 10 files
    for i in 40..50 {
        let modified_file = dir.join(format!("file_{}.txt", i));
        std::fs::write(&modified_file, "Hello, World!").unwrap();
    }
}
