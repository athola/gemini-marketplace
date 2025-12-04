//! The telemetry logger, which provides dual-mode output (human and JSON) and metrics counters.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryFormat {
    Human,
    Json,
    Silent,
}

#[derive(Clone)]
pub struct Telemetry {
    inner: Arc<TelemetryInner>,
}

struct TelemetryInner {
    format: TelemetryFormat,
    writer: Arc<dyn Fn(&str) + Send + Sync>,
    counters: Mutex<TelemetryCounters>,
}

impl Telemetry {
    /// Retrieves the global telemetry handle, initialized from environment variables.
    pub fn global() -> &'static Telemetry {
        static INSTANCE: OnceLock<Telemetry> = OnceLock::new();
        INSTANCE.get_or_init(Telemetry::from_env)
    }

    fn from_env() -> Telemetry {
        let format = env::var("GEMINI_MARKETPLACE_TELEMETRY")
            .ok()
            .and_then(|value| match value.to_ascii_lowercase().as_str() {
                "json" => Some(TelemetryFormat::Json),
                "text" | "human" => Some(TelemetryFormat::Human),
                "off" | "silent" => Some(TelemetryFormat::Silent),
                _ => None,
            })
            .unwrap_or(TelemetryFormat::Human);
        Telemetry::with_writer(format, Arc::new(|line| eprintln!("{line}")))
    }

    /// Constructs a telemetry handle with an explicit format and writer.
    ///
    /// This is useful for tests.
    pub fn with_writer(
        format: TelemetryFormat,
        writer: Arc<dyn Fn(&str) + Send + Sync>,
    ) -> Telemetry {
        Telemetry {
            inner: Arc::new(TelemetryInner {
                format,
                writer,
                counters: Mutex::new(TelemetryCounters::default()),
            }),
        }
    }

    pub fn record_cache_hit(&self, source: &str) {
        self.update_counters(|counters| counters.cache_hits += 1);
        self.emit_event("cache_hit", Some(source), Vec::new());
    }

    pub fn record_cache_miss(&self, source: &str) {
        self.update_counters(|counters| counters.cache_misses += 1);
        self.emit_event("cache_miss", Some(source), Vec::new());
    }

    pub fn record_rate_limit_wait(&self, source: &str) {
        self.update_counters(|counters| counters.rate_limit_waits += 1);
        self.emit_event("rate_limit_wait", Some(source), Vec::new());
    }

    pub fn record_search(&self, keyword: &str) {
        let keyword = keyword.trim();
        if keyword.is_empty() {
            return;
        }
        self.update_counters(|counters| {
            *counters
                .search_terms
                .entry(keyword.to_string())
                .or_default() += 1;
        });
        self.emit_event(
            "search",
            None,
            vec![("keyword", serde_json::Value::String(keyword.to_string()))],
        );
    }

    pub fn record_refresh_queue_depth(&self, depth: usize) {
        self.update_counters(|counters| counters.refresh_queue_depth = depth as u64);
        self.emit_event(
            "refresh_queue_depth",
            None,
            vec![("depth", serde_json::json!(depth))],
        );
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        let guard = self.inner.counters.lock().unwrap();
        guard.snapshot()
    }

    fn update_counters<F>(&self, mut f: F)
    where
        F: FnMut(&mut TelemetryCounters),
    {
        if let Ok(mut guard) = self.inner.counters.lock() {
            f(&mut guard);
        }
    }

    fn emit_event(
        &self,
        event: &'static str,
        source: Option<&str>,
        attributes: Vec<(&str, serde_json::Value)>,
    ) {
        match self.inner.format {
            TelemetryFormat::Silent => {}
            TelemetryFormat::Json => {
                let mut attrs_map = serde_json::Map::new();
                for (key, value) in attributes {
                    attrs_map.insert(key.to_string(), value);
                }
                let payload = serde_json::json!({
                    "event": event,
                    "source": source,
                    "attributes": attrs_map
                });
                if let Ok(line) = serde_json::to_string(&payload) {
                    (self.inner.writer)(&line);
                }
            }
            TelemetryFormat::Human => {
                let mut parts = Vec::new();
                if let Some(src) = source {
                    parts.push(format!("source={src}"));
                }
                for (key, value) in attributes {
                    if let Some(number) = value.as_u64() {
                        parts.push(format!("{key}={number}"));
                    } else if let Some(text) = value.as_str() {
                        parts.push(format!("{key}={text}"));
                    } else {
                        parts.push(format!("{key}={value}"));
                    }
                }
                let suffix = if parts.is_empty() {
                    String::new()
                } else {
                    format!(" {}", parts.join(" "))
                };
                (self.inner.writer)(&format!("[telemetry] {event}{suffix}"));
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetrySnapshot {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub rate_limit_waits: u64,
    pub refresh_queue_depth: u64,
    pub search_terms: BTreeMap<String, u64>,
}

#[derive(Debug, Default)]
struct TelemetryCounters {
    cache_hits: u64,
    cache_misses: u64,
    rate_limit_waits: u64,
    refresh_queue_depth: u64,
    search_terms: HashMap<String, u64>,
}

impl TelemetryCounters {
    fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            rate_limit_waits: self.rate_limit_waits,
            refresh_queue_depth: self.refresh_queue_depth,
            search_terms: self.search_terms.iter().fold(
                BTreeMap::new(),
                |mut acc, (key, value)| {
                    acc.insert(key.clone(), *value);
                    acc
                },
            ),
        }
    }
}
