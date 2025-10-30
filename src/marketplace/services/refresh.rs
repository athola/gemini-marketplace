//! Refresh scheduler service backed by a persisted job queue.

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::{Context, Result};

use crate::marketplace::config::Config;
use crate::marketplace::models::domain::RetryJob;

const QUEUE_FILE: &str = "refresh_queue.json";

#[derive(Clone)]
pub struct RefreshService {
    path: PathBuf,
    jobs: Arc<Mutex<VecDeque<RetryJob>>>,
}

impl RefreshService {
    /// Create a refresh service rooted in the provided configuration directory.
    pub fn new(config: &Config) -> Result<Self> {
        config.ensure_dirs()?;
        let path = config.config_dir().join(QUEUE_FILE);
        let jobs = if path.exists() {
            read_jobs(&path)?
        } else {
            VecDeque::new()
        };
        Ok(Self {
            path,
            jobs: Arc::new(Mutex::new(jobs)),
        })
    }

    /// Queue (or replace) a refresh job and persist the queue to disk.
    pub fn queue_refresh(&self, job: RetryJob) -> Result<()> {
        let mut guard = self.jobs.lock().unwrap();
        guard.retain(|existing| existing.job_id != job.job_id);
        guard.push_back(job);
        sort_jobs(&mut guard);
        persist_jobs(&self.path, &guard)
    }

    /// Return all queued jobs (oldest first).
    pub fn pending_jobs(&self) -> Vec<RetryJob> {
        let guard = self.jobs.lock().unwrap();
        guard.iter().cloned().collect()
    }

    /// Drain jobs scheduled at or before the provided timestamp.
    pub fn drain_due(&self, now: SystemTime) -> Result<Vec<RetryJob>> {
        let mut guard = self.jobs.lock().unwrap();
        let mut ready = Vec::new();
        while let Some(front) = guard.front() {
            if front.scheduled_for <= now {
                ready.push(guard.pop_front().expect("front must exist"));
            } else {
                break;
            }
        }
        if !ready.is_empty() {
            persist_jobs(&self.path, &guard)?;
        }
        Ok(ready)
    }

    /// Return the number of pending jobs.
    pub fn len(&self) -> usize {
        let guard = self.jobs.lock().unwrap();
        guard.len()
    }

    /// Persist current queue state without modifying it.
    pub fn flush(&self) -> Result<()> {
        let guard = self.jobs.lock().unwrap();
        persist_jobs(&self.path, &guard)
    }
}

fn read_jobs(path: &PathBuf) -> Result<VecDeque<RetryJob>> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read refresh queue at {}", path.display()))?;
    let mut jobs: Vec<RetryJob> = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse refresh queue at {}", path.display()))?;
    jobs.sort_by(|a, b| a.scheduled_for.cmp(&b.scheduled_for));
    Ok(jobs.into())
}

fn persist_jobs(path: &PathBuf, jobs: &VecDeque<RetryJob>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create queue directory {}", parent.display()))?;
    }
    let snapshot: Vec<_> = jobs.iter().collect();
    let contents =
        serde_json::to_string_pretty(&snapshot).context("Failed to serialize refresh queue")?;
    fs::write(path, contents)
        .with_context(|| format!("Failed to write refresh queue at {}", path.display()))?;
    Ok(())
}

fn sort_jobs(jobs: &mut VecDeque<RetryJob>) {
    let mut vec: Vec<_> = jobs.drain(..).collect();
    vec.sort_by(|a, b| a.scheduled_for.cmp(&b.scheduled_for));
    *jobs = vec.into();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::{Mutex, OnceLock};
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_temp_home() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let path = temp
            .path()
            .to_str()
            .expect("temp path utf8 for env override")
            .to_string();
        env::set_var("GEMINI_MARKETPLACE_HOME", &path);
        temp
    }

    fn clear_temp_home() {
        env::remove_var("GEMINI_MARKETPLACE_HOME");
    }

    #[test]
    fn queue_and_reload_jobs() {
        let _guard = env_lock().lock().unwrap();
        let temp = with_temp_home();
        let config = Config::new().expect("config");
        let service = RefreshService::new(&config).expect("service");
        assert_eq!(service.len(), 0);

        let job = RetryJob::new(
            "curated",
            "job-1",
            SystemTime::now() + Duration::from_secs(5),
            0,
            "manual refresh",
        );

        service.queue_refresh(job.clone()).expect("queue job");
        assert_eq!(service.len(), 1);

        // Reload service to ensure persistence.
        let service_reloaded = RefreshService::new(&config).expect("reload service");
        assert_eq!(service_reloaded.len(), 1);
        let pending = service_reloaded.pending_jobs();
        assert_eq!(pending[0].job_id, "job-1");

        drop(temp);
        clear_temp_home();
    }

    #[test]
    fn drain_due_returns_ready_jobs() {
        let _guard = env_lock().lock().unwrap();
        let temp = with_temp_home();
        let config = Config::new().expect("config");
        let service = RefreshService::new(&config).expect("service");

        let now = SystemTime::now();
        service
            .queue_refresh(RetryJob::new("curated", "job-now", now, 0, "due"))
            .expect("queue immediate");
        service
            .queue_refresh(RetryJob::new(
                "curated",
                "job-later",
                now + Duration::from_secs(60),
                0,
                "later",
            ))
            .expect("queue future");

        let ready = service
            .drain_due(now + Duration::from_secs(1))
            .expect("drain");
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].job_id, "job-now");
        assert_eq!(service.len(), 1, "remaining job should still be queued");

        drop(temp);
        clear_temp_home();
    }
}
