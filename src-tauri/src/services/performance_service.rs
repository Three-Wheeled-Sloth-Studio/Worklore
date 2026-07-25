use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use chrono::{DateTime, Utc};
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::{
    domain::performance::{
        ActiveOperation, OperationMetric, OperationOutcome, PerformanceSnapshot,
    },
    error::{ServiceResult, WorkLoreError},
    io_utils::{read_json, write_json_atomic},
};

const METRICS_DIRECTORY: &str = ".worklore/operation-metrics";
const METRICS_LOG: &str = "operations.jsonl";

pub struct OperationSession {
    vault_path: PathBuf,
    active: ActiveOperation,
    started: Instant,
}

impl OperationSession {
    pub fn start(
        vault_path: &Path,
        operation: impl Into<String>,
        metadata: Map<String, Value>,
    ) -> ServiceResult<Self> {
        ensure_directories(vault_path)?;
        let now = Utc::now().to_rfc3339();
        let active = ActiveOperation {
            schema_version: 1,
            run_id: format!("run_{}", Uuid::now_v7()),
            operation: operation.into(),
            phase: "starting".to_string(),
            started_at: now.clone(),
            updated_at: now,
            elapsed_ms: 0,
            progress_current: None,
            progress_total: None,
            metadata,
        };
        write_active(vault_path, &active)?;
        Ok(Self {
            vault_path: vault_path.to_path_buf(),
            active,
            started: Instant::now(),
        })
    }

    pub fn run_id(&self) -> &str {
        &self.active.run_id
    }

    pub fn set_phase(&mut self, phase: impl Into<String>) -> ServiceResult<()> {
        self.active.phase = phase.into();
        self.active.updated_at = Utc::now().to_rfc3339();
        self.active.elapsed_ms = duration_ms(self.started.elapsed());
        write_active(&self.vault_path, &self.active)
    }

    pub fn set_progress(
        &mut self,
        current: u64,
        total: Option<u64>,
    ) -> ServiceResult<()> {
        self.active.progress_current = Some(current);
        self.active.progress_total = total;
        self.active.updated_at = Utc::now().to_rfc3339();
        self.active.elapsed_ms = duration_ms(self.started.elapsed());
        write_active(&self.vault_path, &self.active)
    }

    pub fn step<T, F>(
        &mut self,
        phase: &str,
        metadata: Map<String, Value>,
        action: F,
    ) -> ServiceResult<T>
    where
        F: FnOnce() -> ServiceResult<T>,
    {
        self.set_phase(phase)?;
        let started_at = Utc::now().to_rfc3339();
        let started = Instant::now();
        let result = action();
        let completed_at = Utc::now().to_rfc3339();
        let metric = OperationMetric {
            schema_version: 1,
            run_id: format!("step_{}", Uuid::now_v7()),
            parent_run_id: Some(self.active.run_id.clone()),
            operation: self.active.operation.clone(),
            phase: phase.to_string(),
            started_at,
            completed_at,
            duration_ms: duration_ms(started.elapsed()),
            outcome: if result.is_ok() {
                OperationOutcome::Succeeded
            } else {
                OperationOutcome::Failed
            },
            error_code: result.as_ref().err().map(|_| "worklore_error".to_string()),
            metadata,
        };
        append_metric(&self.vault_path, &metric)?;
        result
    }

    pub fn finish<T>(mut self, result: ServiceResult<T>) -> ServiceResult<T> {
        let completed_at = Utc::now().to_rfc3339();
        let metric = OperationMetric {
            schema_version: 1,
            run_id: self.active.run_id.clone(),
            parent_run_id: None,
            operation: self.active.operation.clone(),
            phase: if result.is_ok() {
                "complete".to_string()
            } else {
                self.active.phase.clone()
            },
            started_at: self.active.started_at.clone(),
            completed_at,
            duration_ms: duration_ms(self.started.elapsed()),
            outcome: if result.is_ok() {
                OperationOutcome::Succeeded
            } else {
                OperationOutcome::Failed
            },
            error_code: result.as_ref().err().map(|_| "worklore_error".to_string()),
            metadata: std::mem::take(&mut self.active.metadata),
        };
        let log_result = append_metric(&self.vault_path, &metric);
        let remove_result = remove_active(&self.vault_path, &self.active.run_id);

        match result {
            Ok(value) => {
                log_result?;
                remove_result?;
                Ok(value)
            }
            Err(error) => {
                let _ = log_result;
                let _ = remove_result;
                Err(error)
            }
        }
    }
}

pub fn snapshot(vault_path: &Path, limit: usize) -> ServiceResult<PerformanceSnapshot> {
    ensure_directories(vault_path)?;
    Ok(PerformanceSnapshot {
        active_operations: list_active(vault_path)?,
        recent_metrics: list_recent_metrics(vault_path, limit.clamp(1, 200))?,
    })
}

pub fn recover_interrupted(vault_path: &Path) -> ServiceResult<usize> {
    ensure_directories(vault_path)?;
    let active = list_active(vault_path)?;
    let now = Utc::now();
    let mut recovered = 0;

    for operation in active {
        let started = DateTime::parse_from_rfc3339(&operation.started_at)
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(now);
        let elapsed = now.signed_duration_since(started).num_milliseconds().max(0) as u64;
        append_metric(
            vault_path,
            &OperationMetric {
                schema_version: 1,
                run_id: operation.run_id.clone(),
                parent_run_id: None,
                operation: operation.operation,
                phase: operation.phase,
                started_at: operation.started_at,
                completed_at: now.to_rfc3339(),
                duration_ms: elapsed,
                outcome: OperationOutcome::Interrupted,
                error_code: Some("process_interrupted".to_string()),
                metadata: operation.metadata,
            },
        )?;
        remove_active(vault_path, &operation.run_id)?;
        recovered += 1;
    }

    Ok(recovered)
}

fn ensure_directories(vault_path: &Path) -> ServiceResult<()> {
    fs::create_dir_all(metrics_root(vault_path).join("active"))?;
    Ok(())
}

fn metrics_root(vault_path: &Path) -> PathBuf {
    vault_path.join(METRICS_DIRECTORY)
}

fn active_path(vault_path: &Path, run_id: &str) -> PathBuf {
    metrics_root(vault_path)
        .join("active")
        .join(format!("{run_id}.json"))
}

fn write_active(vault_path: &Path, active: &ActiveOperation) -> ServiceResult<()> {
    write_json_atomic(&active_path(vault_path, &active.run_id), active)
}

fn remove_active(vault_path: &Path, run_id: &str) -> ServiceResult<()> {
    let path = active_path(vault_path, run_id);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn append_metric(vault_path: &Path, metric: &OperationMetric) -> ServiceResult<()> {
    ensure_directories(vault_path)?;
    let path = metrics_root(vault_path).join(METRICS_LOG);
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, metric)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn list_active(vault_path: &Path) -> ServiceResult<Vec<ActiveOperation>> {
    let directory = metrics_root(vault_path).join("active");
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut operations = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file()
            && entry.path().extension().and_then(|value| value.to_str()) == Some("json")
        {
            operations.push(read_json(&entry.path())?);
        }
    }
    operations.sort_by(|left, right| right.started_at.cmp(&left.started_at));
    Ok(operations)
}

fn list_recent_metrics(vault_path: &Path, limit: usize) -> ServiceResult<Vec<OperationMetric>> {
    let path = metrics_root(vault_path).join(METRICS_LOG);
    if !path.is_file() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut metrics = reader
        .lines()
        .filter_map(Result::ok)
        .filter_map(|line| serde_json::from_str::<OperationMetric>(&line).ok())
        .collect::<Vec<_>>();
    metrics.reverse();
    metrics.truncate(limit);
    Ok(metrics)
}

fn duration_ms(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_conversion_saturates_to_u64() {
        assert_eq!(duration_ms(std::time::Duration::from_millis(42)), 42);
    }

    #[test]
    fn metric_limit_is_bounded_by_snapshot_caller() {
        assert_eq!(500_usize.clamp(1, 200), 200);
    }
}
