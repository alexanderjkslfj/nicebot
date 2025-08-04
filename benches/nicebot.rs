use nicebot::NiceBot;
use std::{fs, hint::black_box, str::FromStr};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[inline]
fn get_robots() -> impl Iterator<Item = (String, String)> {
    fs::read_dir("./test-data")
        .unwrap()
        .into_iter()
        .filter_map(|item| item.ok())
        .filter_map(|entry| {
            let name = String::from(entry.file_name().to_string_lossy());
            let path = entry.path();
            let Ok(content) = fs::read_to_string(path) else {
                return None;
            };
            Some((name, content))
        })
}

fn from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("NiceBot::from_str");
    for (name, content) in get_robots() {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            content.as_str(),
            |b, r| {
                b.iter(|| {
                    let _ = NiceBot::from(r);
                });
            },
        );
    }
    group.finish();
}

fn trim(c: &mut Criterion) {
    let mut group = c.benchmark_group("NiceBot::trim");
    for (name, content) in get_robots() {
        let robot = NiceBot::from(content);
        group.bench_with_input(BenchmarkId::from_parameter(name), &robot, |b, r| {
            b.iter_batched(
                || r.clone(),
                |mut robot_clone| {
                    robot_clone.trim();
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn check(c: &mut Criterion) {
    let mut group = c.benchmark_group("NiceBot::check short");
    for (name, content) in get_robots() {
        let robot = NiceBot::from(content);
        group.bench_with_input(BenchmarkId::from_parameter(name), &robot, |b, r| {
            b.iter(|| {
                r.check(black_box(""));
                r.check(black_box("/"));
                r.check(black_box("*"));
                r.check(black_box("/abcdefghijklmnopqrstuvwxyz"));
                r.check(black_box(
                    "/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z",
                ));
            });
        });
    }
    group.finish();

    let mut long_string_slashy = String::new(); // string like /0/1/2/3/4/...
    let mut long_string = String::from_str("/").unwrap(); // string like /01234567...
    for i in 0..1000 {
        long_string_slashy.push('/');
        let str = i.to_string();
        long_string.push_str(&str);
        long_string_slashy.push_str(&str);
    }

    let mut group = c.benchmark_group("NiceBot::check long");
    for (name, content) in get_robots() {
        let robot = NiceBot::from(content);
        group.bench_with_input(BenchmarkId::from_parameter(name), &robot, |b, r| {
            b.iter(|| {
                r.check(black_box(&long_string));
                r.check(black_box(&long_string_slashy));
            });
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().plotting_backend(criterion::PlottingBackend::Plotters);
    targets = from_str, trim, check
}
criterion_main!(benches);
