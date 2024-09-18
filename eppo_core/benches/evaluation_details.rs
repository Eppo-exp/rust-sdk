use std::collections::HashMap;
use std::fs::File;

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use eppo_core::{
    eval::{get_assignment, get_assignment_details},
    Configuration, SdkMetadata,
};

fn criterion_benchmark(c: &mut Criterion) {
    let flags =
        serde_json::from_reader(File::open("../sdk-test-data/ufc/flags-v1.json").unwrap()).unwrap();
    let configuration = Configuration::from_server_response(flags, None);
    let now = Utc::now();
    let meta = SdkMetadata {
        name: "test",
        version: "0.1.0",
    };

    {
        let mut group = c.benchmark_group("new-user-onboarding");
        group.throughput(Throughput::Elements(1));
        let attributes = HashMap::new();
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("rollout");
        group.throughput(Throughput::Elements(1));
        let attributes = [("country".to_owned(), "US".into())].into();
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("json-config-flag");
        group.throughput(Throughput::Elements(1));
        let attributes = [].into();
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("json-config-flag"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("json-config-flag"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("numeric-one-of");
        group.throughput(Throughput::Elements(1));
        let attributes = [("number".to_owned(), 2.0.into())].into();
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("numeric-one-of"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("numeric-one-of"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("regex-flag");
        group.throughput(Throughput::Elements(1));
        let attributes = [("email".into(), "test@gmail.com".into())].into();
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("regex-flag"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("regex-flag"),
                    black_box("subject1"),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                    &meta,
                )
            })
        });
        group.finish();
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().noise_threshold(0.02);
    targets = criterion_benchmark);
criterion_main!(benches);
