# Operation Instrumentation

## Goal

WorkLore will perform some tasks locally that may take long enough for the user to walk away, forget what they started, and return suspicious of the laptop.

Instrumentation must make those operations inspectable without collecting the private content being processed.

## Initial Coverage

Source import is instrumented by phase:

```text
import_source
  hash_source
  check_duplicate
  copy_source
  extract_text
  write_extraction_cache
  scan_private_entities
  scan_named_projects
  write_source_metadata
```

Resume candidate extraction is also instrumented:

```text
extract_resume_candidates
  parse_and_write_candidates
```

Future long-running operations should use the same pattern, including:

- OCR
- Git repository discovery and commit clustering
- Bulk resume or source imports
- Local embedding and indexing
- Ollama provider calls
- Gemini provider calls
- Story synthesis and evidence audits
- Vault migrations and backups

## Background Execution

Source import and resume candidate extraction run through Tauri's blocking-task runtime rather than on the UI command thread.

This matters for two reasons:

1. The desktop interface remains responsive while local parsing or scanning runs.
2. The three-second instrumentation poll can continue reading active operation state during the task.

New long-running commands should follow this pattern rather than placing a large synchronous operation directly inside a Tauri command handler.

## Storage

Instrumentation is stored in the user vault under rebuildable application data:

```text
.worklore/operation-metrics/
  operations.jsonl
  active/
    run_<id>.json
```

`operations.jsonl` is append-only diagnostic history. Active operation records are updated atomically as phases change.

## Recorded Fields

Operations may record:

- Operation and phase names
- Start and completion timestamps
- Duration in milliseconds
- Outcome
- Parent and child run IDs
- Process ID for interrupted-run recovery
- File size
- Extracted character count
- Progress counts
- Provider and model identifiers
- Error category

## Prohibited Fields

Instrumentation must not contain:

- Resume or source text
- Interview questions or answers
- Exact private entity names
- Provider prompts or responses
- API keys or credential material
- File contents

Canonical record IDs may be added when useful for diagnosis, but should be omitted unless the workflow needs them.

## Interrupted Operations

An active operation file includes the process ID that created it. When a vault opens, WorkLore converts active records from a different process into completed metrics with outcome `interrupted`.

Records owned by the current process are left alone so a routine workspace refresh cannot declare its own task dead while that task is still working.

## User Interface

The workspace polls local instrumentation every three seconds and displays:

- The current operation and phase
- Live elapsed time calculated from the recorded start timestamp
- Progress when available
- The slowest recent phases
- Recent top-level outcomes

The blocking operation layer also shows the current phase and elapsed time.

## Retention

The first build retains local metrics until the user deletes them or the vault is backed up without rebuildable data.

A later increment should add:

- Configurable retention by age or row count
- Export to CSV or JSON for performance comparison
- Machine profile metadata such as CPU, memory, and GPU class without unique hardware identifiers
- Aggregate percentiles by operation and phase
- Comparison between the primary workstation and backup laptop
