# Vault Format

## Purpose

A WorkLore vault is a user-owned folder containing original source files and portable canonical records. The application may build indexes and caches around those records, but it must never make the database the only usable copy of a user's work.

## Initial Layout

```text
WorkLoreVault/
  vault.json

  sources/
    resumes/
    job-descriptions/
    writing-samples/
    interview-transcripts/
    git-snapshots/
    other/
    metadata/

  roles/
  candidates/
  stories/
  interviews/
  jobs/
  voice/

  privacy/
    entity-registry.json
    review-items/
    scan-log/

  exports/
    manual-workspaces/
    markdown/
    json/

  backups/

  .worklore/
    index.sqlite
    extraction-cache/
    provider-logs/
    operation-journal/
```

## Canonical And Rebuildable Data

### Canonical

- `vault.json`
- Imported source files
- Source metadata JSON
- Role JSON
- Candidate JSON
- Story Markdown and JSON
- Interview JSON
- Job analysis JSON
- Voice profile records
- Private Entity Registry records
- Entity review decisions
- User-created exports

### Rebuildable

- SQLite indexes
- Extracted text caches
- Embeddings
- Search indexes
- Provider request estimates
- Temporary workspace packages
- Derived summaries that retain canonical inputs

Deleting `.worklore/index.sqlite` must not remove durable user content.

## Story Storage

Each story has:

```text
stories/
  story_<id>.md
  story_<id>.json
```

The JSON record is the canonical structured contract. The Markdown record is the canonical human-readable rendering.

Both files carry the same story ID and revision. A save operation writes them as one logical transaction. When a write is interrupted, WorkLore should preserve the previous valid pair and recover from the operation journal.

## Imported Sources

Source files are copied into the appropriate source-type folder by default.

A source metadata file is stored separately:

```text
sources/metadata/source_<id>.json
```

Metadata includes:

- Original filename
- Optional original path hint
- Copied vault-relative path
- Content hash
- Byte size
- Media type
- Import time
- Extraction state
- Privacy scan state
- Provenance

The original source file is never modified by WorkLore.

## Duplicate Handling

Before copying a source, WorkLore computes SHA-256.

When the hash already exists, the user should be offered:

- Use the existing source record
- Import another labeled reference to the same stored content
- Cancel

WorkLore must not create duplicate stored bytes silently.

## Paths

Canonical records store vault-relative paths using forward slashes.

Machine-local recent-vault settings may store absolute paths outside the vault. Canonical records must not depend on those paths for portability.

## IDs

Canonical IDs use stable prefixed values:

- `vault_`
- `source_`
- `role_`
- `candidate_`
- `story_`
- `interview_`
- `job_`
- `requirement_`
- `voice_`
- `entity_`
- `review_`
- `evidence_`
- `claim_`

The first implementation may use UUIDv7 or ULID payloads after the prefix. IDs are opaque to users.

## Schema Versions

Every canonical JSON record includes `schemaVersion`.

Migrations must:

1. Detect the existing schema version.
2. Write a recoverable backup or operation journal entry.
3. Produce the new record without changing the durable ID.
4. Validate the new record.
5. Replace the old record atomically.
6. Record the migration result.

## Atomic Writes

For canonical text and JSON files:

1. Write to a temporary file in the destination directory.
2. Flush and close the temporary file.
3. Validate it where applicable.
4. Rename it over the target using the safest atomic operation available on the platform.
5. Retain the prior version until the logical multi-file save completes.

Long-running imports and migrations must show a blocking operation state in the UI.

## Backups

Automatic live synchronization is out of scope.

The first version should support:

- Manual vault export or copy
- A backup command that produces a timestamped archive or folder copy
- Excluding rebuildable `.worklore` data by default
- Optional inclusion of caches for diagnostic support

Backup and restore must never include provider credentials because credentials live in the operating system credential store.

## Machine-Local Application State

Recent vault paths, window state, and credential lookup keys belong in the application's machine-local settings, not inside the vault unless a setting is explicitly portable.

## Vault Validation

Opening a vault should validate:

- `vault.json` exists and matches a supported schema
- Required canonical directories exist or can be created
- The vault is writable for editing operations
- Entity registry identity matches the vault
- No incomplete operation journal entry requires recovery

A vault with newer unsupported schema versions should open read-only when practical rather than being modified blindly.
