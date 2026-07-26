# Private Entity Registry

## Purpose

The Private Entity Registry gives each WorkLore vault a durable understanding of sensitive names and identifiers. It prevents the application from rebuilding mappings for every prompt and gives users explicit control over what can leave their machine.

The registry covers traditional personally identifiable information and broader sensitive business context, including employers, clients, projects, products, systems, repositories, and internal identifiers.

## Authoritative Records

Canonical registry records live in the vault:

```text
privacy/
  entity-registry.json
  review-items/
  scan-log/
```

The registry JSON is authoritative. SQLite indexes, normalized lookup tables, embeddings, and scan caches are rebuildable.

## Stable Identity

Every entity has:

- A stable `entityId`
- A canonical name
- Zero or more aliases
- An entity type
- A sensitivity setting
- A stable public token
- Occurrences linked to source records
- Optional relationships to other entities
- Revision history

The `entityId` never changes.

The public token never changes while the entity remains active. Tokens are never recycled. Deleting or merging an entity must not cause a future entity to inherit its old token.

Example:

```text
FINRA -> [EMPLOYER_1]
Kinections -> [PROJECT_2]
```

## Entity Types

Initial types:

- employer
- client
- project
- product
- system
- repository
- person
- location
- email
- phone
- url
- account
- identifier
- organization
- user_defined

Entity type and sensitivity are independent. A public employer can own a private project. A person's public name can still be marked `never_send_to_cloud` by the user.

## Sensitivity Levels

### public

May be sent to any configured provider unless blocked by another rule.

### private

Redacted when the vault or request privacy mode is `redact`.

### ask_before_cloud

Cloud preflight requires explicit user confirmation when the entity is present.

### never_send_to_cloud

Cloud requests containing the entity must redact it. A per-request override cannot bypass this setting.

## Scan Triggers

An entity scan runs when:

- A source is imported
- Extracted source text changes
- A story is created
- A story is materially edited
- An interview answer is saved
- A candidate is created or edited
- A job description is added or updated
- Writing samples or explicit voice instructions are added
- A provider result is imported
- A local repository snapshot is created or refreshed
- Text is pasted into a managed record

A save may complete before the scan completes, but the record remains visibly marked `pending` or `needs_review`.

## Scan Pipeline

1. Normalize the text while retaining stable character offsets or fragment locators.
2. Match confirmed aliases and tokens from the existing registry.
3. Run deterministic detectors for emails, phone numbers, URLs, account-like identifiers, and user-defined patterns.
4. Run named entity extraction for organizations, people, locations, products, projects, and systems.
5. Score the suggested entity type.
6. Search existing entities using aliases, context, relationships, and semantic similarity.
7. Score identity matches.
8. Auto-link only when rules and confidence permit.
9. Create review items for ambiguous or high-risk results.
10. Create provisional entities for likely new entities when allowed.
11. Store occurrences and scan metadata.
12. Mark the record complete or needs review.

The scan must be repeatable. Re-scanning the same unchanged content should not produce duplicate entities or occurrences.

## Confidence Model

WorkLore stores three distinct scores:

### Extraction confidence

How likely is the text span to be an entity rather than ordinary prose?

### Type confidence

How likely is the suggested entity type correct?

### Identity match confidence

How likely is the occurrence the same real-world entity as an existing registry entry?

Initial thresholds:

- `0.92` or higher: eligible for automatic linking when no high-risk rule blocks it
- `0.65` through `0.919999`: user review required
- Below `0.65`: probable new entity or ignored low-confidence extraction, depending on risk and detector type

Deterministic exact alias matches can link with a confidence of `1.0` unless the alias itself is disputed or scoped.

Thresholds are implementation defaults, not part of the durable entity identity. They may become advanced settings later.

## User Review

A review card should show:

- Matched text
- Nearby context
- Suggested type
- Best existing matches
- Confidence scores
- Plain-language match reasons
- Risk level

Primary actions:

- Same entity
- Related, but separate
- Create a new entity
- Change entity type
- Use this as the canonical name
- Ignore this term
- Dismiss this occurrence

Example:

```text
Is "Kinections" the same project as
"LLM tool to pre-screen investor complaints"?
```

When the user selects `Same entity`, the matched text becomes a confirmed alias of the existing entity.

When the user selects `Related, but separate`, WorkLore creates a new entity and a relationship between the two.

When the user selects `Ignore this term`, WorkLore records a normalized ignored-term rule with either record or vault scope.

## Provisional Entities

A likely new entity may be created with status `provisional` when:

- Extraction confidence is sufficient
- No strong existing match exists
- The entity is not a low-risk generic phrase
- The record can remain in a review-needed state

Provisional entities receive stable internal IDs and tokens, but should remain visually distinguishable until confirmed.

## Merge Behavior

Merging entities must:

1. Select one surviving entity.
2. Preserve the surviving entity ID and token.
3. Add non-duplicate aliases and occurrences from the retired entity.
4. Reconcile relationships.
5. Mark the retired entity as merged.
6. Store `redirectToEntityId` on the retired entity.
7. Add the retired token to the registry redirect table.
8. Update indexes and affected record references.
9. Preserve an audit event that can support reversal.

The retired token remains valid as an input token but is not emitted in new exports.

## Split Behavior

Splitting an incorrect merge must:

1. Create or reactivate the separated entity.
2. Assign a new token if its original token was never previously issued.
3. Restore its retired token when the split reverses a prior merge and doing so is unambiguous.
4. Move selected aliases, occurrences, and relationships.
5. Re-scan affected records.
6. Preserve an audit event.

A split must never silently rewrite old external exports.

## Redaction Modes

### Private exact names

Used for local records and explicitly allowed cloud requests.

### Stable private tokens

Used for redacted provider requests and structured exports.

Example:

```text
[EMPLOYER_1] built [PROJECT_2] to help pre-screen investor complaints.
```

### Descriptive anonymization

Used for public-facing content.

Example:

```text
At a financial-services regulator, an LLM-assisted intake tool helped reduce manual complaint triage.
```

Descriptive replacements are explicit registry fields or user-approved generated text. WorkLore must not assume a public description is safe merely because it is vague.

## Request Redaction

Before a cloud request:

1. Collect the selected records and fragments.
2. Verify that their privacy scans cover the current content revision.
3. Resolve token redirects.
4. Replace entities based on sensitivity and the active request mode.
5. Block unresolved `never_send_to_cloud` or high-risk items.
6. Generate a preflight summary.
7. Send only the redacted request body after approval requirements are satisfied.

Replacement must operate on recorded occurrences where possible. Blind global string replacement is a fallback and must avoid replacing substrings inside unrelated words.

## Response Rehydration

Provider responses using stable tokens may be rehydrated locally.

Rehydration rules:

- Resolve retired tokens through redirects.
- Replace only recognized complete tokens.
- Preserve unknown tokens and flag them for review.
- Do not rehydrate public outputs unless the user selects exact-name mode.
- Store the original tokenized provider result when needed for provenance, subject to provider log retention preferences.

## Cloud Blocking Rules

Cloud execution is blocked when selected content contains:

- An unresolved critical review item
- An occurrence of a `never_send_to_cloud` entity that could not be safely replaced
- A stale privacy scan for a record revision
- A failed redaction integrity check

The user may override ordinary `private` or `ask_before_cloud` handling where allowed. The user may not override `never_send_to_cloud` without first changing the entity setting.

## Audit Events

Registry operations should produce local audit events for:

- Entity creation
- Alias addition or rejection
- Sensitivity change
- Canonical name change
- Relationship change
- Auto-link
- User resolution
- Merge
- Split
- Token redirect
- Re-scan
- Redaction preflight

Audit events should identify affected records without duplicating full sensitive bodies.

## Testing Requirements

Tests must cover:

- Stable token allocation
- No token reuse
- Exact alias matching
- Ambiguous alias review
- Merge redirects
- Split reversal
- Record revision invalidating an old scan
- High-risk cloud blocking
- Deterministic redaction
- Token-aware rehydration
- Idempotent re-scan
- The `Kinections` ambiguity fixture
