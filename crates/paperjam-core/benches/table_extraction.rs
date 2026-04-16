//! Criterion microbench for table extraction over the fixture corpus.
//!
//! Run with: `cargo bench -p paperjam-core --bench table_extraction`
//!
//! Each synthetic PDF fixture under `tests/fixtures/tables/` becomes a bench group.
//! The document and page are parsed outside the measured section so the bench only
//! measures `table::extract_tables` itself — that's the code subsequent phases will
//! change.

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use paperjam_core::document::Document;
use paperjam_core::table::{extract_tables, TableExtractionOptions};

fn fixtures_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../../tests/fixtures/tables")
}

fn collect_fixtures() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let dir = fixtures_dir();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        eprintln!(
            "skipping bench: fixtures dir not found at {}",
            dir.display()
        );
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn bench_extract(c: &mut Criterion) {
    let fixtures = collect_fixtures();
    let opts = TableExtractionOptions::default();

    let mut group = c.benchmark_group("table_extraction");
    for path in &fixtures {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        // Parse the document once and keep its pages alive for the whole bench run.
        let doc = match Document::open(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("skipping {name}: failed to open ({e:?})");
                continue;
            }
        };
        let mut pages = Vec::new();
        for n in 1..=doc.page_count() as u32 {
            if let Ok(p) = doc.page(n) {
                pages.push(p);
            }
        }
        group.throughput(Throughput::Elements(pages.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&name), &pages, |b, pages| {
            b.iter(|| {
                for page in pages.iter() {
                    let _ = extract_tables(page, &opts);
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_extract);
criterion_main!(benches);
