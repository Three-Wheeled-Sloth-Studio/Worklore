# Vault Format and Canonical Domain Contract

Status: accepted canonical domain contract
Updated: 2026-09-13

## Purpose

A WorkLore vault is a user-owned, portable folder containing canonical structured professional memory, original source material, privacy mappings, and exportable history.

This contract replaces the prototype assumption that every durable structured record must remain canonical as paired JSON or Markdown files. The prototype files remain valid migration inputs. They do not constrain the refocused domain model.

## Storage decision

For the refocused product, SQLite is the recommended canonical store for structured records, relationships, lineage, audit events, revisions, experiments, and analytics.

Canonical source bytes and attachments remain ordinary files in the vault filesystem.

Human-readable export is a required portability and recovery capability, but exported JSON, JSONL, and Markdown are snapshots of canonical state rather than a second live database that must be kept transactionally synchronized on every edit.

This split provides:

- reliable transactions across related records;
- queryable many-to-many relationships without duplicated ID arrays;
- exact revision and analytics joins;
- durable migration support;
- simpler audit/version behavior;
- a portable vault that still works without WorkLore-hosted storage or sync.

## Target vault layout

```text
WorkLoreVault/
  vault.json

  data/
    worklore.sqlite

  sources/
    originals/
    attachments/

  exports/
    snapshots/
    markdown/
    json/

  backups/

  .worklore/
    extraction-cache/
    search-cache/
    embeddings/
    provider-logs/
    operation-journal/
```

`vault.json`, `data/worklore.sqlite`, original imported source bytes, user-created attachments, and the Private Entity Registry state represented in the database are canonical.

Everything under `.worklore` is rebuildable or diagnostic unless a later contract explicitly says otherwise.

Provider credentials remain outside the vault in the operating-system credential store.

## Canonical identity and lifecycle rules

Canonical IDs are opaque, stable, prefixed identifiers. New records should use a UUIDv7 or equivalent sortable random payload after the prefix.

Existing durable IDs are preserved when the migrated concept retains the same meaning. When a prototype concept changes meaning, WorkLore creates the new canonical ID and records the legacy ID in migration lineage rather than preserving an accidental prefix or shape.

All mutable canonical records carry at least:

- stable ID;
- record type;
- lifecycle status;
- created timestamp;
- updated timestamp;
- integer row revision for optimistic concurrency;
- origin/provenance metadata.

Archiving is preferred to destructive deletion for records that may participate in provenance, publication history, voice evidence, or analytics. Explicit user-requested purge remains possible but must be auditable and must preserve referential integrity.

Lifecycle state and domain maturity are separate where both matter. For example, a Story may be `active` while its maturity is `developing`.

## Canonical domain model

| Concept | ID prefix | Lifecycle or maturity | Canonical purpose |
| --- | --- | --- | --- |
| Story | `story_` | lifecycle: `active`, `archived`; maturity: `developing`, `evidence_rich`, `ready_to_use` | Durable account of something the user experienced or did. A Story does not require a Role. |
| Story Seed | `seed_` | `captured`, `developing`, `converted`, `dismissed`, `archived` | Incomplete memory or fragment worth preserving and potentially developing. |
| Proof Point | `proof_` | `candidate`, `confirmed`, `retired` | Reusable factual result, decision, artifact, number, qualitative outcome, tool use, or lesson. |
| Topic Candidate | `topic_` | `captured`, `exploring`, `ready`, `drafted`, `parked`, `retired` | Durable idea the user may have credible reason to discuss. It exists independently of any Post. |
| Theme | `theme_` | `emerging`, `active`, `retired` | Recurring area of demonstrated expertise or professional interest. |
| Inspiration | `inspiration_` | `saved`, `processed`, `archived` | External material used to stimulate concepts, questions, counterpoints, or connections. It is not Evidence about the user. |
| Target Context | `target_` | `active`, `stale`, `archived` | Opportunity or audience context such as a job description, public profile, or audience example. It informs ideation, not keyword stuffing. |
| Voice Evidence | `voice_evidence_` | `pending`, `eligible`, `rejected`, `retired` | A specifically approved sample that may inform canonical voice. Eligibility is explicit and provenance-gated. |
| Core Voice | `voice_` | `proposed`, `active`, `superseded` | Versioned stable author-identity traits derived only from eligible Voice Evidence and explicit user guidance. |
| Tone Mode | `tone_` | `active`, `disabled`, `retired` | Intentional register that changes expression without becoming a separate identity. |
| Voice Direction | `voice_direction_` | `proposed`, `accepted`, `completed`, `retired` | User-approved direction for deliberate voice evolution. |
| Writing Rule | `rule_` | `proposed`, `active`, `disabled`, `retired` | Generation or review constraint kept separate from author identity. |
| Audience Model | `audience_` | `draft`, `active`, `retired` | Structured reader archetype used for Audience Lens interpretation. |
| Post | `post_` | `idea`, `drafting`, `challenge`, `review`, `approved`, `published`, `archived` | Professional content artifact and workflow container. Exact text lives in immutable Revision records. |
| Revision | `revision_` | immutable snapshot; approval state recorded separately | Exact content version with parent lineage, content hash, origin actor, provider run where relevant, and change metadata. |
| Experiment | `experiment_` | `planned`, `active`, `completed`, `cancelled` | Explicit content hypothesis linked to one or more Posts or Publications. |
| Performance Record | `performance_` | immutable import snapshot; may be superseded by a later snapshot | Imported analytics for a specific Publication and exact published Revision. |
| Source | `source_` | `active`, `archived` | Neutral provenance record for imported or captured material and its original bytes or captured text. |
| Evidence Record | `evidence_` | `candidate`, `confirmed`, `rejected`, `retired` | Specific support for a factual claim, Story, or Proof Point, with locator and provenance. |

## Supporting records that remain useful

The refocused model also retains supporting concepts where they add real value:

- Role remains an optional career-context record with `role_` IDs. It can link to Stories, Sources, and entities but does not own them and is never mandatory for Story creation.
- Interview remains a durable guided-development session with `interview_` IDs. New sessions target a Story Seed or Story directly instead of requiring a resume candidate.
- Private Entity records preserve current `entity_` IDs and stable public tokens.
- Provider Run records capture model/provider execution metadata without retaining private request bodies by default.
- Publication records use `publication_` IDs to represent the manual act of publishing an exact approved Revision.
- Audit Events use `audit_` IDs and record material state transitions and provenance-affecting changes.
- Migration Map records preserve legacy IDs and source paths when concepts are transformed.

## Relationship model

High-value relationships are many-to-many and should be represented by typed associative tables rather than duplicated arrays stored on both records.

At minimum the persistence layer must support:

- Story <-> Story Seed lineage;
- Story <-> Proof Point;
- Story <-> Theme;
- Story <-> Role;
- Proof Point <-> Evidence Record;
- Evidence Record <-> Source;
- Topic Candidate <-> Story;
- Topic Candidate <-> Proof Point;
- Topic Candidate <-> Theme;
- Topic Candidate <-> Inspiration;
- Topic Candidate <-> Target Context;
- Voice Evidence <-> Source or Revision;
- Core Voice version <-> Voice Evidence;
- Post <-> Topic Candidate;
- Post <-> Story;
- Post <-> Proof Point;
- Post <-> Theme;
- Post <-> Inspiration;
- Post <-> Target Context;
- Post <-> Audience Model;
- Post <-> Experiment;
- Publication -> Post + exact published Revision;
- Performance Record -> Publication + exact published Revision.

Low-value generic associations such as tags may use reusable link tables. Relationships that enforce product invariants should use typed tables and foreign keys.

## Provenance contract

`Source` is neutral origin metadata. Evidence, Inspiration, Target Context, and Voice Evidence are separate semantic records or usage decisions and must never collapse into one source-type enum.

A single imported file may participate in more than one workflow only through explicit role records. For example, a user-authored article may be a Source, Inspiration for a new Topic Candidate, and separately approved Voice Evidence. Those uses remain independently auditable.

Derived records capture:

- creation actor: `user`, `import`, `system`, `model`, or `migration`;
- source record IDs or parent revision IDs;
- provider/model run ID when model assistance was involved;
- creation timestamp;
- content hash for immutable text snapshots;
- explicit user confirmation or approval when required.

Provider output is provenance, not evidence about the user.

## Voice-training eligibility invariant

Raw model output can never become canonical Voice Evidence merely because it exists in the vault.

Every Revision records its origin and parentage. Revisions whose content is raw provider/model output are permanently marked `voice_training_prohibited`.

Voice Evidence may be created only when at least one of these conditions is true:

1. The material is genuinely user-authored or user-dictated and the user accepts it as representative.
2. The material is a user-edited descendant of generated text and the user explicitly accepts that edited text as representative.
3. The material is the exact human-approved final version of a Post and the user allows final approved work to contribute to voice.
4. The material is explicit user guidance about desired or undesired voice behavior.

Approval of a later Revision never changes the eligibility of an earlier raw model Revision. The raw draft remains excluded forever.

Inspiration and Target Context are never automatically eligible for voice learning, even when their source text happens to resemble the user.

### Implemented Voice Evidence provenance boundary

`task-031` implements the first canonical Voice Evidence boundary in schema version 6. A writing-sample Source may create one idempotent governed `voice_evidence_` record for the full attributable source text, but the record begins `pending` with authorship `unknown` and approval `unreviewed`.

The implemented authorship states are `unknown`, `user_authored`, `user_edited_model`, `model_generated`, and `external_author`. Once explicitly asserted, authorship provenance is immutable through the normal review path. Only `user_authored` and `user_edited_model` can become `eligible`, and only after explicit approval. Attempts to approve `model_generated` or `external_author` material result in durable rejection with an explicit prohibition reason.

Eligibility lifecycle is `pending`, `eligible`, `rejected`, or `retired`; approval state is `unreviewed`, `approved`, `rejected`, or `revoked`. Retiring preserves the Source and Voice Evidence lineage and cannot be reversed through the normal review path. Candidate creation and eligibility changes append audit events and increment the mutable record revision. Target Context Sources are blocked from Voice Evidence, and Inspiration remains semantically separate.

### Implemented Core Voice and intentional-range boundary

The provider-free Core Voice foundation is implemented in schema version 7. Core Voice versions use `proposed`, `active`, and `superseded` lifecycle, with transactional activation so the initial single-user vault has at most one active version while preserving superseded history. Traits are separate queryable records and must retain attributable provenance to eligible Voice Evidence and/or explicit user guidance.

Rejected or retired Voice Evidence cannot be newly attached to a proposed trait. If evidence that already supports historical Core Voice is later retired or rejected, WorkLore surfaces that provenance as invalid for review rather than silently rewriting the historical version.

Tone Modes (`active`, `disabled`, `retired`), Voice Directions (`proposed`, `accepted`, `completed`, `retired`), and Writing Rules (`proposed`, `active`, `disabled`, `retired`) are persisted independently so intentional expression, desired evolution, and behavioral constraints do not collapse into observed identity. Accepting a Voice Direction or changing a Tone Mode/Rule never mutates Core Voice automatically.

Provider-assisted analysis remains downstream of this boundary. A provider may propose attributable traits, but provider output is review material rather than authoritative identity and must enter Core Voice only through the same explicit proposed-version and provenance rules.

## Draft, edit, approval, and publication lineage

A Post is a workflow container. A Revision is an immutable text snapshot.

The normal lineage is:

`angle/idea -> model or user draft Revision -> user-edited Revision(s) -> approved Revision -> manual Publication -> Performance Record(s)`

Each Revision stores a `parent_revision_id`, content hash, origin actor, and creation metadata.

Approval identifies one exact Revision. Manual publication records one exact Revision and the user-entered publication metadata.

If the user changes the text while publishing outside WorkLore, WorkLore should capture or paste back the exact published text as a new user/import Revision and attach the Publication to that Revision instead of pretending the previously approved text was published unchanged.

Performance Records must link to the Publication and its exact Revision. Analytics must never join to "the current Post text" because the current draft may have changed after publication.

## Evidence and standing

Proof Points and substantive claims link to Evidence Records, not directly to generic Source metadata.

Evidence Records may represent:

- a source fragment;
- a user-confirmed memory;
- an interview answer;
- a project artifact;
- a Git commit or diff summary;
- another explicit factual record.

Model inference may be retained in provenance or review notes, but it is not Evidence until the user confirms it or another supported source establishes it.

A Post can reference Inspiration and Target Context without those records becoming proof of the user's experience.

## Privacy and confidentiality

Private canonical records may retain real employer, client, person, project, product, system, repository, location, and metric details.

Private Entity Registry IDs and stable tokens remain durable. Entity occurrences may link to any canonical record or Revision.

Public-use checks are tied to the exact Revision content hash and the Private Entity Registry revision used for review.

A Post cannot move to `approved` for public use, and a Publication cannot be recorded, unless the exact candidate Revision has a passing confidentiality review or an explicit user override permitted by policy.

When WorkLore produces public-safe wording, the transformed Revision derives from the private Revision and keeps the same factual Evidence lineage. Confidentiality transformation changes disclosure, not truth.

## Audit and version expectations

SQLite transactions protect multi-record changes.

Mutable domain records use optimistic row revisions. Material text artifacts use immutable Revision records.

Append-only Audit Events are required for:

- migration;
- merge or split operations;
- evidence confirmation or rejection;
- voice-evidence eligibility changes;
- Core Voice activation or supersession;
- privacy-rule changes affecting public output;
- Post approval;
- Publication recording or correction;
- analytics import;
- destructive purge.

The audit log need not capture every keystroke. It must capture changes that affect provenance, identity, privacy, publication state, or later interpretation.

## Migration and reuse from the prototype

| Prototype concept | Refocused mapping |
| --- | --- |
| Vault document | Preserve vault ID and user-facing metadata in `vault.json`; migrate structured state to `data/worklore.sqlite`. |
| SourceDocument | Preserve `source_` IDs, copied bytes, hashes, extraction state, provenance, and privacy state. Add semantic role records instead of expanding `SourceType` indefinitely. |
| Resume source | Preserve as Source. Resume processing becomes optional `Seed from resume` and may create Story Seeds, Roles, Evidence Records, and Stories. |
| Job description | Preserve source bytes and provenance. Create Target Context. Existing job/requirement analysis becomes target-context concepts, not Evidence about the user. |
| Writing sample | Preserve as Source. Create Voice Evidence only after provenance establishes user authorship and the user approves eligibility. |
| StoryCandidate | Migrate to Story Seed where still useful. Use a new `seed_` ID and retain the legacy `candidate_` ID in migration lineage. Converted candidates may become archived/converted seeds linked to their Story. |
| Story | Preserve `story_` IDs and factual content. Map prototype status to lifecycle plus maturity. Remove mandatory Role coupling. |
| Nested Story evidence | Promote to global Evidence Records and preserve existing `evidence_` IDs where semantics remain valid. |
| Story claims | Confirmed or user-estimated reusable claims may become candidate Proof Points. Unsupported or model-inferred claims remain provenance/review material until established. |
| Role | Preserve `role_` IDs as optional career context. Existing reverse `storyIds` arrays are replaced by relationship rows. |
| Interview | Preserve `interview_` IDs and turn history. Remap candidate targets to Story Seeds; future interviews may target Story Seeds or Stories directly. |
| Voice profile explicit instructions | Migrate to Writing Rules, preserving text and enabled/priority semantics. |
| Voice profile observations | Import as proposed Core Voice traits requiring review rather than silently accepting historical inference as identity. |
| Voice profile sample refs | Import as pending Voice Evidence candidates. Do not mark eligible until user authorship and approval are established. |
| Private Entity Registry | Preserve entity IDs, stable tokens, aliases, redirects, review decisions, and occurrence history. |
| Provider/manual workspace artifacts | Preserve as provider-run or audit provenance where useful. Model text remains in Revision lineage and is not automatically Voice Evidence. |

Migration must be recoverable and non-destructive. No prototype JSON or Markdown record should be deleted until the new database has been validated and a backup or export exists.

## Portability and export

Portability does not require WorkLore-hosted sync.

Canonical database paths and Source records use vault-relative paths. Machine-local recent-vault settings may keep absolute paths outside the vault.

A portable export snapshot should be deterministic and schema-versioned and may include:

- a manifest with vault ID, export time, schema version, and content hashes;
- JSON or JSONL domain records;
- Markdown renderings for Stories, Topics, Voice summaries, and Posts where useful;
- copied original Source files when requested;
- publication and analytics metadata;
- privacy/entity data only when the user explicitly includes private records.

Export does not expose provider credentials.

For backup or vault copy, WorkLore should checkpoint SQLite WAL state before copying and should treat concurrent multi-machine editing through a synced filesystem as unsupported unless a future sync contract explicitly adds it.

## Prototype compatibility after canonical migration

The canonical SQLite persistence boundary and non-destructive migration seam are now implemented. Legacy JSON/Markdown records remain valid migration and compatibility inputs where existing prototype workflows still use them, but new canonical domain concepts belong behind the canonical SQLite service boundary.

Do not extend the old per-record JSON/Markdown layout with new canonical concepts. Preserve legacy files until migration is validated and a backup or export exists, and keep compatibility paths explicit rather than treating the prototype layout as a second live canonical database.

## Validation expectations

Persistence implementation that follows this contract must eventually demonstrate:

1. stable IDs survive migration;
2. Story creation does not require a Role or resume;
3. Source semantic roles remain distinct;
4. raw model text cannot become Voice Evidence through any normal code path;
5. exact approved and published Revision lineage is preserved;
6. Performance Records join to the exact published Revision;
7. privacy review is tied to exact public-use content;
8. a copied vault opens without machine-local absolute-path dependencies;
9. human-readable export can be regenerated from canonical state;
10. no WorkLore-hosted service is required.
