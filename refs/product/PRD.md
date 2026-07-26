# WorkLore Product Requirements

## Product Summary

WorkLore is a local-first desktop application that helps people recover useful career stories from the evidence they already have.

The application imports resumes, job descriptions, writing samples, and local Git history. It identifies potential accomplishments, asks focused follow-up questions, and turns the answers into discrete, reusable story records linked to roles and source evidence.

The first release is designed for one primary user and a small group of technically comfortable testers. It must work without a hosted WorkLore account and must keep the user's durable career records in a local vault they control.

## Core Job To Be Done

When I need to explain what I have accomplished, I want to recover the useful facts buried across my resume, documents, and project history, then turn those facts into credible stories I can reuse for interviews, resumes, and professional content.

## Product Principles

1. The user's local vault is authoritative.
2. The application must distinguish confirmed facts, user estimates, model inferences, and unsupported claims.
3. Generated claims must retain provenance back to source material or interview answers.
4. Private records and public outputs are separate concerns.
5. Cloud use is optional, visible, and controlled by the user.
6. Reversible actions should be easy to perform and easy to undo.
7. The application controls workflow state. Language models perform bounded tasks through structured contracts.
8. The interface should keep current state and the next useful action obvious.

## Primary Users

### Primary tester

A technically comfortable professional using WorkLore on one or more personal computers, but without live synchronization between them.

### Early test group

Approximately five technically comfortable users who can install a desktop test build, Ollama, or configure a Gemini API key.

## MVP Outcomes

A user can:

1. Create or open a local WorkLore vault.
2. Import source documents by copying them into the vault.
3. Extract candidate stories from a resume.
4. Review, ignore, merge, split, or interview a candidate.
5. Answer one focused interview question at a time.
6. Produce a discrete story linked to a role and its evidence.
7. Edit and validate the resulting story.
8. Maintain a durable Private Entity Registry for employers, clients, projects, people, systems, and other sensitive entities.
9. Redact private entities with stable vault-level tokens before sending content to Gemini or a manual external workspace.
10. Use Ollama locally, a user-supplied Gemini API key, or a manual workspace package.
11. Export human-readable Markdown and structured JSON.

## MVP Scope

### Vault management

- Create a vault in a user-selected folder.
- Open an existing vault.
- Remember recently opened vaults in machine-local application settings.
- Support multiple vaults without optimizing the interface around simultaneous use.
- Copy imported source files into the vault by default.
- Detect duplicate source content using a cryptographic content hash.
- Keep canonical records in portable Markdown and JSON.
- Treat SQLite, embeddings, extraction output, and other indexes as rebuildable machine-local state.
- Do not provide live synchronization.

### Source ingestion

Initial source types:

- Resume
- Job description
- Writing sample
- Interview transcript or notes
- Other supporting document
- Local Git repository activity

Initial document formats:

- Plain text
- Markdown
- PDF with extractable text
- DOCX

When text cannot be extracted, WorkLore must report that state and must not pretend the source was understood.

### Resume candidate extraction

For each resume bullet or meaningful claim, WorkLore should identify:

- Employer and role context
- Claimed action
- Claimed outcome
- Metrics
- Tools, systems, and methods
- Skills or competencies
- Missing context
- Potential relationship to an existing story

The user can mark a candidate as:

- Ready to interview
- Saved for later
- Merged with another candidate or story
- Split into multiple candidates
- Ignored
- Unsupported or no longer accurate

### Job description analysis

Job descriptions are requirement lenses, not historical evidence.

WorkLore should extract:

- Required capabilities
- Preferred capabilities
- Domain expectations
- Leadership expectations
- Tools and platforms
- Implied business problems

It should match those requirements against existing stories and create new interview candidates only where there is plausible related or transferable experience.

### Guided interview

The interview workspace presents one primary question at a time.

The user can:

- Answer
- Skip
- Mark that they do not remember
- Ask why the question matters
- Request memory prompts or possible categories
- Mark an answer as uncertain or estimated
- Split the candidate into multiple stories
- Merge the candidate with an existing story
- Pause and resume later

Questions should be generated in rounds of no more than three, with the default experience presenting them individually.

The interview completeness model includes:

- Context
- Problem or opportunity
- User responsibility
- Constraints
- Actions and decisions
- Alternatives considered
- Stakeholders
- Tools and systems
- Outcomes
- Metrics and evidence
- Lessons learned
- Operating philosophy
- Reusable themes
- Disclosure restrictions

### Story records

A story represents one discrete accomplishment, decision, failure, lesson, or meaningful professional episode.

A story can link to:

- One or more roles
- Employers or clients
- Projects, products, systems, and repositories
- Source fragments
- Interview answers
- Job requirements
- Related stories

A story stores private exact names. Public outputs can use stable tokens or approved descriptive replacements.

Required story sections:

- Title
- Summary
- Situation and context
- Problem or opportunity
- Responsibilities
- Constraints
- Actions and decisions
- Tools and systems
- Stakeholders
- Outcomes
- Metrics
- Evidence and confidence
- Lessons learned
- Operating philosophy
- Skills demonstrated
- Disclosure rules
- Source provenance
- Revision history

### Private Entity Registry

Every meaningful content addition or edit must trigger an entity scan.

The registry must support:

- Stable internal entity identifiers
- Stable redaction tokens such as `[EMPLOYER_1]`
- Canonical names
- Aliases
- Entity type
- Sensitivity level
- Source occurrences
- Relationships to other entities
- Extraction confidence
- Type confidence
- Existing-entity match confidence
- User review state
- Merge, split, alias, and redirect history

Low-confidence or ambiguous matches must be presented to the user for resolution. For example, WorkLore may ask whether `Kinections` is the same project as `LLM tool to pre-screen investor complaints`.

Tokens are vault-local, stable, and never recycled. When entities merge, obsolete tokens redirect to the surviving entity.

### Provider support

#### Ollama

- Detect a locally running Ollama service.
- List available models.
- Run a structured-output capability test.
- Keep all selected content local.

#### Gemini bring-your-own-key

- Store the API key in the operating system credential store.
- Never place the key in the vault or repository.
- Use structured JSON output contracts.
- Show the selected model, privacy mode, included sources, and approximate request size before the first cloud request in a session.
- Record local request metadata without storing sensitive request or response bodies by default.

#### Manual workspace package

- Export selected source excerpts, story context, voice guidance, privacy-safe content, task instructions, and a response schema.
- Support pasting or importing the structured result back into WorkLore.

### Cloud privacy preference

The user has a persistent preference controlling whether private identifiers may be sent to cloud providers.

Modes:

- Redact private identifiers before cloud use
- Include private identifiers in cloud requests

The preference controls the default. Every cloud session or request can override it.

A preflight view must show:

- Provider and model
- Privacy mode
- Sources included
- Identifiers replaced
- Unresolved privacy review items
- Approximate request size

Cloud actions must pause when relevant content has unresolved high-risk entity matches.

### Voice profile

The first release stores:

- Explicit voice and tone instructions
- Writing samples
- Inferred voice observations
- Representative excerpts
- Confidence and source references

Explicit instructions override inferred observations.

Integrated LinkedIn generation is not required for the first story-bank release. WorkLore should support a manual export package that makes this workflow practical to test.

### Local Git evidence

The first Git increment should support:

- Registering one local repository
- Discovering repositories beneath a selected parent folder
- Selecting repositories to include
- Reading commit history through the installed Git command line
- Grouping likely related commits by branch, issue reference, time, file area, and message similarity
- Creating reviewable story candidates from meaningful clusters
- Linking commits and changed files as evidence

Private GitHub account access is a later increment.

## Evidence Model

Every claim used in a story must be classified as one of:

- Confirmed fact
- User estimate
- Model inference
- Unsupported claim

Model inference must never be silently promoted to confirmed fact.

A finalized story may contain estimates when clearly labeled. Unsupported claims should remain visible to the user but should be excluded from generated public or job-facing output by default.

## Nonfunctional Requirements

### Local ownership

- Durable user records remain in the selected vault.
- The vault remains useful without WorkLore installed because canonical records are readable Markdown and JSON.
- Rebuildable indexes can be deleted and recreated without loss of durable records.

### Privacy and security

- API keys use the operating system credential store.
- The application never writes secrets to logs.
- Cloud requests are explicit and reviewable.
- Private entity replacements are deterministic within a vault.
- User vaults and imported documents must not be stored inside the application repository.

### Reliability

- Copy and save operations must be atomic where practical.
- Interrupted imports must not leave a source marked complete.
- Schema versions must be stored with canonical records.
- Migrations must preserve original records or create a recoverable backup.

### Performance

- The application should reach a usable first screen quickly.
- Long-running import, scan, provider, and repository operations must show progress or clear indeterminate activity.
- The primary workspace must remain readable without excessive scrolling or oversized empty surfaces.

### Accessibility

- Full keyboard operation for primary workflows.
- Visible focus states.
- Labels for icon-only controls.
- Status must not rely on color alone.
- Content should remain usable at common Windows display scaling levels.

## Out Of Scope For Initial Story Bank Release

- Live synchronization
- Hosted WorkLore accounts
- Included paid LLM credits
- Automatic LinkedIn publishing or scheduling
- LinkedIn analytics ingestion
- GitHub account OAuth and private repository APIs
- Bundled local models
- Custom audio recording or transcription
- Mobile applications
- Multi-user collaboration inside one vault

## Initial Release Sequence

### Release 0.1: Vault foundation

- Desktop shell
- Vault create and open
- Canonical folder structure
- Source copy and metadata
- Duplicate detection
- Entity registry foundation
- Basic story editor

### Release 0.2: Story extraction and interview

- Resume parsing
- Candidate extraction
- Candidate triage
- Interview state machine
- Story synthesis and evidence audit
- Ollama and Gemini provider paths

### Release 0.3: Job lens

- Job requirement extraction
- Story matching
- Evidence gap review
- Job-specific story collections

### Release 0.4: Repository archaeology

- Local repository registration
- Parent-folder discovery
- Commit clustering
- Repository-derived story candidates

### Release 0.5: Voice and content bridge

- Writing sample analysis
- Explicit and inferred voice profile
- Public anonymization
- Manual LinkedIn workspace export and result import

## MVP Acceptance Summary

The story-bank MVP is ready for the first external testers when a new user can create a vault, import a resume, resolve detected private entities, select a resume bullet, complete a resumable interview, produce and edit a discrete evidence-backed story, close and reopen the application without losing state, and export the story as readable Markdown and valid JSON using either Ollama or a user-supplied Gemini key.