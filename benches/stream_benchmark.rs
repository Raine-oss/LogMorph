// Benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use logmorph::engine::AggregationEngine;
use logmorph::parser::LogStreamReader;
use std::io::Cursor;

// Benchmark Data Generator

fn generate_synthetic_log(line_count: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(line_count * 80);
    for i in 0..line_count {
        if i % 100 == 0 {
            data.extend_from_slice(
                b"[12:00:00 ERROR]: Could not pass event PlayerMoveEvent to TestPlugin v1.0\n\
org.bukkit.event.EventException: null\n\
\tat org.bukkit.plugin.java.JavaPluginLoader.execute(JavaPluginLoader.java:100)\n\
\tat com.test.plugin.MoveListener.onMove(MoveListener.java:42)\n\
\tat net.minecraft.server.MinecraftServer.tick(MinecraftServer.java:500)\n"
            );
        } else {
            data.extend_from_slice(b"[12:00:00 INFO]: Server tick performance normal\n");
        }
    }
    data
}

// Benchmark Suite

fn bench_stream_parser(c: &mut Criterion) {
    let raw_log = generate_synthetic_log(10_000);
    let size_bytes = raw_log.len() as u64;

    let mut group = c.benchmark_group("stream_parsing");
    group.throughput(Throughput::Bytes(size_bytes));

    group.bench_function("parse_10k_lines", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&raw_log));
            let reader = LogStreamReader::new(cursor);
            let mut engine = AggregationEngine::new();

            for event in reader {
                engine.feed_event(event.unwrap());
            }

            black_box(engine.stats().total_lines);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_stream_parser);
criterion_main!(benches);
