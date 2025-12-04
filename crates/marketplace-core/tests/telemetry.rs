use std::sync::{Arc, Mutex};

use gemini_marketplace::telemetry::{Telemetry, TelemetryFormat};

type WriterBuffer = Arc<Mutex<Vec<String>>>;
type WriterFn = Arc<dyn Fn(&str) + Send + Sync>;

fn memory_writer() -> (WriterBuffer, WriterFn) {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer_buffer = buffer.clone();
    let writer = Arc::new(move |line: &str| {
        writer_buffer
            .lock()
            .expect("lock writer buffer")
            .push(line.to_string());
    });
    (buffer, writer)
}

#[test]
fn telemetry_counts_events_and_emits_json() {
    let (buffer, writer) = memory_writer();
    let telemetry = Telemetry::with_writer(TelemetryFormat::Json, writer);

    telemetry.record_cache_hit("curated");
    telemetry.record_cache_miss("community");
    telemetry.record_rate_limit_wait("curated");
    telemetry.record_search("search term");
    telemetry.record_refresh_queue_depth(3);

    let snapshot = telemetry.snapshot();
    assert_eq!(snapshot.cache_hits, 1);
    assert_eq!(snapshot.cache_misses, 1);
    assert_eq!(snapshot.rate_limit_waits, 1);
    assert_eq!(snapshot.refresh_queue_depth, 3);
    assert_eq!(snapshot.search_terms.get("search term"), Some(&1));

    let entries = buffer.lock().unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.contains("\"event\":\"cache_hit\"")),
        "expected cache_hit event in JSON output"
    );
}
