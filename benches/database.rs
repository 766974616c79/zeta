use criterion::{Criterion, criterion_group, criterion_main};
use zeta::Database;

fn criterion_benchmark(c: &mut Criterion) {
    let mut database = Database::default();
    c.bench_function("database insert", |b| b.iter(||       database.insert(String::from(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vivamus suscipit justo a magna dapibus, in porta ipsum auctor. Proin in ornare est. Vivamus vestibulum felis orci, at mattis nisl consequat in. Sed laoreet pretium urna, id volutpat libero vulputate ac. Aliquam tempus ex ac dolor dignissim ornare. Nullam vel nisl leo. Pellentesque sed justo tortor. Donec id quam arcu.",
    ))));

    database.insert(String::from("hello"));

    c.bench_function("database get", |b| b.iter(|| database.get("hello")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
