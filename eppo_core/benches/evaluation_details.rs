use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use eppo_core::ufc::UniversalFlagConfig;
use eppo_core::{
    eval::{get_assignment, get_assignment_details},
    Configuration, SdkMetadata,
};

fn criterion_benchmark(c: &mut Criterion) {
    let flags = UniversalFlagConfig::from_json(
        SdkMetadata {
            name: "test",
            version: "0.1.0",
        },
        std::fs::read("../sdk-test-data/ufc/flags-v1.json").unwrap(),
    )
    .unwrap();
    let configuration = Configuration::from_server_response(flags, None);
    let now = Utc::now();

    {
        let mut group = c.benchmark_group("new-user-onboarding");
        group.throughput(Throughput::Elements(1));
        let attributes = Arc::new(HashMap::new());
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("rollout");
        group.throughput(Throughput::Elements(1));
        let attributes = Arc::new([("country".into(), "US".into())].into());
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("new-user-onboarding"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("json-config-flag");
        group.throughput(Throughput::Elements(1));
        let attributes = Arc::new([].into());
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("json-config-flag"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("json-config-flag"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("numeric-one-of");
        group.throughput(Throughput::Elements(1));
        let attributes = Arc::new([("number".into(), 2.0.into())].into());
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("numeric-one-of"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("numeric-one-of"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.finish();
    }

    {
        let mut group = c.benchmark_group("regex-flag");
        group.throughput(Throughput::Elements(1));
        let attributes = Arc::new([("email".into(), "test@gmail.com".into())].into());
        group.bench_function("get_assignment", |b| {
            b.iter(|| {
                get_assignment(
                    black_box(Some(&configuration)),
                    black_box("regex-flag"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
                )
            })
        });
        group.bench_function("get_assignment_details", |b| {
            b.iter(|| {
                get_assignment_details(
                    black_box(Some(&configuration)),
                    black_box("regex-flag"),
                    black_box(&"subject1".into()),
                    black_box(&attributes),
                    black_box(None),
                    black_box(now),
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
