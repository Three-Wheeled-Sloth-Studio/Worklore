# WorkLore Product Requirements Document

Status: locked product direction
Updated: 2026-09-12

## 1. Product definition

WorkLore is a local-first professional narrative and content intelligence system.

It helps a user capture what they have done and what they think, develop those inputs into durable career stories and topic candidates, learn and deliberately evolve the user's authentic voice, generate credible professional content without generic AI writing patterns, and learn from audience response over time.

The product is not primarily a resume tool. Resume ingestion may bootstrap the system, but the durable product is the user's professional memory, voice, evidence, topic graph, publication history, and accumulated learning about what resonates.

Primary loop:

`Capture -> Understand -> Develop -> Connect -> Draft -> Challenge -> Publish manually -> Measure -> Learn`

## 2. Primary user and product framing

The primary user is a career-active professional building a visible professional identity. Active job search is an important mode, but it does not define the product.

The user's likely goals include:

- preserving useful career memories before details are lost;
- developing evidence-backed stories for interviews, professional writing, and career materials;
- capturing promising post topics without needing to draft immediately;
- writing LinkedIn-appropriate posts that sound recognizably like the user;
- avoiding generic AI voice, repetitive templates, unsupported thought leadership, and engagement bait;
- connecting timely external topics to areas where the user has legitimate experience or a useful point of view;
- learning which topics, tones, structures, audiences, and publication times produce useful engagement;
- maintaining discretion when private career evidence is transformed into public content.

## 3. Product principles

### 3.1 Local canonical ownership

The user's durable professional memory is local and user-controlled.

Canonical data lives on the user's machine. WorkLore may call user-authorized local or third-party services for bounded generation, research, or retrieval, but WorkLore itself is not a hosted service and does not require a WorkLore account.

SQLite or a similar durable local store is acceptable as canonical or indexed local persistence where it improves reliability and queryability. Human-readable export remains important. The architecture must not depend on WorkLore-hosted storage.

A vault may be copied or placed in a user-managed synced folder, but WorkLore does not promise proprietary vault synchronization in the accepted roadmap.

### 3.2 Human publication boundary

WorkLore does not automatically publish, schedule, or post content to LinkedIn or other social platforms.

The user remains the final editor and publisher. This is an intentional product boundary, not merely deferred integration work.

WorkLore may prepare copyable or exportable content. Public posting always requires explicit human review and action outside the application.

Automatic social publishing, social scheduling integration, and autonomous engagement are excluded from the accepted roadmap. They may be reconsidered only through a future product decision.

### 3.3 Evidence before eloquence

The system should prefer a smaller number of supported claims over polished but generic prose.

Every substantive claim about the user should be traceable to documented experience, a user-confirmed statement, or another explicit source. WorkLore must not invent outcomes, metrics, employers, tools, biographical details, or authority.

### 3.4 Productive resistance to slop

WorkLore is not neutral about output quality.

The product should resist being used as a generic content factory. It may warn, challenge, deprioritize, or recommend discarding material that is repetitive, unsupported, derivative, generic, over-produced, or primarily engagement bait.

This resistance is advisory rather than an absolute ban. The user remains in control, but WorkLore should be willing to say that a draft adds little, repeats recent material, lacks standing, or reads unlike the user.

### 3.5 Stable identity with controlled range

The goal is not one frozen tone. WorkLore should learn a stable author identity that can express an intentional range.

A serious post, analytical post, reflective post, and whimsical post may differ substantially in tone while still sounding like the same person.

Voice drift can be harmful or intentional. The system must distinguish accidental movement toward generic AI patterns from deliberate user-directed evolution.

### 3.6 Private accuracy, public discretion

Private story records should preserve real company names, tools, metrics, and context when the user provides them.

Public-facing drafts may transform or omit private entities according to user rules. Accuracy remains in the private record. Discretion is applied downstream.

Confidentiality transformation is a hard requirement.

## 4. Canonical domain concepts

The durable domain should support at least the following concepts. Exact schemas are an implementation decision, but these semantic distinctions are product requirements.

### Story

A durable account of something the user actually experienced or did. Stories may be incomplete and can mature over time.

Suggested maturity states:

`seed -> developing -> evidence-rich -> ready-to-use`

### Story seed

An incomplete memory or fragment worth preserving and potentially interviewing the user about later.

### Proof point

A reusable factual result, decision, artifact, number, qualitative outcome, tool use, or lesson supported by evidence.

### Topic candidate

An idea the user may have a credible reason to discuss publicly. Topic candidates persist independently of drafts.

### Theme

A recurring area of demonstrated expertise or professional interest connecting stories, topics, and posts.

### Voice evidence

User-authored or user-approved material used to learn how the user actually communicates.

### Core voice

Relatively stable traits that make output recognizably the user.

### Tone mode

An intentional register such as serious, analytical, conversational, reflective, dry/funny, or whimsical.

Tone modes are not separate identities. They are controlled expressions of the same author identity.

### Voice direction

A user-approved direction in which WorkLore should encourage the voice to evolve over time.

### Writing rules

Behavioral constraints for generated content. These are separate from voice identity.

### Inspiration

External material the user finds interesting, such as a LinkedIn post, article, paper, quote, URL, or excerpt. Inspiration may stimulate concepts, questions, counterpoints, or connections, but is not evidence about the user and must not be used to imitate another author's prose.

### Target context

Material describing what a desired audience or opportunity may care about, including job descriptions, the user's own LinkedIn profile, selected public profiles, or audience examples.

Target context supports ideation. It must not force keyword stuffing or overfit every post to a single job, person, or role.

### Audience model

A structured description of a reader group whose likely interpretation of a post is useful to evaluate, such as a VP Product, technical product peer, recruiter, or healthcare executive.

### Post

A drafted, approved, or published piece of professional content connected to source stories, topics, proof points, themes, target context, and inspiration where applicable.

### Revision

A durable version of a post or other generated artifact, including AI draft, user edits, and final approved text.

### Experiment

A deliberate content hypothesis such as testing shorter posts, different tone, alternate closing behavior, or different publication times.

### Performance record

Imported engagement or LinkedIn analytics connected to a published post and its content metadata.

### Source/evidence record

Provenance for claims, stories, voice evidence, external inspiration, or target context.

## 5. Source classes must remain distinct

WorkLore must distinguish three source roles:

### Evidence

Supports factual claims about the user and the user's work.

Examples: career story, user-confirmed memory, project notes, role records.

### Inspiration

Stimulates something the user may want to think or write about.

Examples: article, paper, LinkedIn post, quote, news item.

Inspiration does not establish that the user has expertise or firsthand experience.

### Target context

Suggests topics, concepts, skills, vocabulary, or concerns that may matter to an intended audience.

Examples: target job description, the user's LinkedIn profile, selected public profiles, target-industry material.

Target context is for ideation and audience understanding. It must not become an automatic optimization target or cause posts to read like resumes or ATS output.

## 6. Primary workflows

### 6.1 Capture anything useful

Capture is the preferred front door.

The user should be able to quickly enter or paste:

- a memory;
- a story fragment;
- a post idea;
- a question;
- a proof point;
- a URL;
- an excerpt;
- a job description;
- a writing sample;
- a piece of feedback;
- a note about something worth revisiting.

Capture must save first and classify second. The product should tolerate ambiguity and incomplete material rather than forcing a form before saving.

Voice dictation should be supported through the operating system or another low-friction input path without making built-in transcription a prerequisite.

### 6.2 Develop a story

WorkLore should help turn a seed into a useful story by identifying gaps and asking targeted questions.

Useful story coverage includes, as applicable:

- what was broken, at stake, or high-risk;
- what the user decided and why;
- what the user actually did;
- what got in the way;
- what was delivered;
- metrics or qualitative proof;
- what standard, process, or system remained afterward;
- what was learned.

STAR is a useful completeness model, not a required printed structure. Finished stories should read naturally rather than as labeled form fields.

### 6.3 Develop a topic

A topic can begin from the user's own thought, a story, target context, inspiration, or current news.

WorkLore should connect it to relevant themes and evidence, identify whether the user has standing to discuss it, and propose genuinely different possible angles before drafting prose.

### 6.4 Save inspiration

The user should eventually be able to pin a post, article, paper, URL, quote, or excerpt with provenance and a personal note.

WorkLore may extract concepts, questions, counterpoints, and possible connections to the user's experience.

It must not silently turn another author's wording into the user's voice model or imitate another author's prose.

### 6.5 Ingest target context

A target job description is a high-value target-context source.

WorkLore should mine it for topics, concepts, responsibilities, skills, tensions, and language that may suggest useful areas to discuss. It should surface connections to the user's existing stories and themes without overfitting content to the job description.

The same conceptual path may later support the user's own LinkedIn profile, selected public professional profiles, or audience examples.

### 6.6 Generate angles before posts

Before drafting, WorkLore should propose multiple genuinely different approaches across tone, structure, opening strategy, and angle.

The system should include common, moderately uncommon, and unexpected but defensible approaches. It should avoid both mode collapse and novelty for novelty's sake.

### 6.7 Audience Lens

Before publication, WorkLore should be able to evaluate what signal a draft likely sends to relevant reader archetypes.

Audience Lens should primarily explain interpretation, strengths, ambiguities, and missing signals. It should not default to rewriting the same post four ways.

Target context may inform audience models, but the system must communicate uncertainty rather than pretending to know an individual reader's private preferences.

### 6.8 Draft and challenge

A draft should be evaluated for:

- voice fit;
- selected tone-mode fit;
- evidence and standing;
- confidentiality;
- unsupported claims;
- generic or empty advice;
- AI-default phrasing and structures;
- angle similarity to recent drafts;
- proof-point reuse;
- theme repetition;
- sentence and opening-pattern repetition;
- inappropriate engagement bait;
- audience signal;
- factual provenance.

The user can override warnings, but the product should explain why it is pushing back.

### 6.9 Edit, approve, and archive

WorkLore must preserve the AI draft, meaningful revisions, and exact final human-approved text.

The final published version is the highest-value writing sample for future voice learning.

The product should retain a robust audit trail connecting the final artifact to its source stories, topics, proof points, inspiration, target context, provider/model where relevant, revisions, approval, publication metadata entered by the user, and later analytics.

### 6.10 Import analytics and learn

Initial analytics support should use user-imported LinkedIn analytics rather than direct LinkedIn integration.

The system should connect performance to metadata such as:

- topic;
- theme;
- story;
- angle;
- tone mode;
- length;
- post type;
- proof points;
- publication time;
- audience intent;
- experiment.

Recommendations must consider sample size and confidence. Small samples are signals, not laws.

Useful outcomes include identifying which content tends to drive profile views, saves, relevant comments, or other user-selected measures, then proposing what to increase, hold, reduce, or test next.

## 7. Voice intelligence contract

Voice learning is a first-class product requirement.

### 7.1 Allowed canonical voice evidence

Canonical voice may learn from:

- original user-authored writing the user marks or accepts as representative;
- user-authored comments or other communication explicitly ingested for voice learning;
- user-approved final posts;
- user edits to generated material;
- spoken/dictated material where the user intends it to represent natural expression;
- explicit user feedback about desired or undesired voice traits.

### 7.2 Raw AI drafts do not train voice

This is a hard rule.

Unedited AI-generated drafts, candidate text, provider responses, rejected drafts, and other model-originated prose must not strengthen, alter, or contaminate the canonical voice model.

A generated draft may become voice evidence only through human transformation or explicit approval as final authored output, with provenance retained.

The system must preserve enough provenance to distinguish user-originated, model-originated, edited, approved, and published text.

### 7.3 Edit-delta learning

User edits are valuable evidence.

WorkLore should eventually learn recurring preferences such as removed phrases, preferred qualification, humor patterns, sentence rhythm, opening behavior, closing behavior, banned words, and repeated corrections.

It should surface proposed voice-learning changes for user acceptance rather than silently rewriting identity from a single edit.

### 7.4 Controlled evolution

Voice audits should detect accidental drift toward generic AI patterns while allowing deliberate user-directed evolution.

The user may intentionally ask WorkLore to become more concise, warmer, less formal, more playful, more analytical, or otherwise evolve. Desired direction should be explicit and distinguishable from observed core voice.

### 7.5 Tone diversity

A healthy content portfolio may include serious, analytical, reflective, conversational, playful, whimsical, or darkly humorous material when appropriate.

Tone diversity is not itself an anti-pattern. Repeated reliance on one template or a tone that no longer resembles the user is an anti-pattern.

## 8. Anti-slop and writing-quality contract

WorkLore should use deterministic and model-assisted review rather than unreliable claims that text is "human" or "AI-written."

The product should prefer a Writing Pattern Linter and portfolio-level checks.

Examples of useful checks include:

- repeated declarative-hook structure;
- repeated rhetorical-question openings;
- hook -> context -> lesson -> question used as the dominant template;
- repetitive numbered or bulleted list structure;
- overuse of the same metric or anecdote;
- repeated sentence rhythm;
- banned words or phrases;
- generic advice without first-party evidence;
- opinion presented as expertise without standing;
- motivational-poster or influencer framing;
- excessive or irrelevant hashtags;
- forced closing questions;
- repeated engagement bait;
- inappropriate certainty;
- excessive em dash or punctuation patterns according to the user's rules;
- batch mode collapse where several posts differ only superficially.

WorkLore should track patterns across recent drafts and published posts, not only inside a single draft.

## 9. Confidentiality and privacy

Private Entity Registry behavior remains strategically useful, but privacy is infrastructure rather than the user's primary navigation model.

Requirements:

- private canonical records can retain real employer, client, person, project, product, and metric names;
- public transformations follow explicit confidentiality rules;
- stable tokens may be used for external AI operations where appropriate;
- provider-bound content receives a local privacy preflight;
- the user can define entities that must never appear in public output;
- public phrasing may substitute functional descriptions such as "a regulated medical-device program" where appropriate;
- transformations must preserve factual meaning and avoid implying false anonymity or relationships.

Vault-level encryption remains desired before broad public distribution. OS disk encryption is an acceptable interim recommendation during early technical testing.

## 10. Discovery and external-awareness requirements

### 10.1 News radar

News scanning is a high-value feature candidate after the story, theme, and voice foundations exist.

The objective is not a generic trending feed. WorkLore should identify current events where the user may have something credible and useful to add.

A future ranking model should consider at least:

`relevance + credibility + audience fit + freshness + angle novelty - saturation - risk`

The system should be able to conclude that an event is relevant but the user lacks enough firsthand experience or differentiated perspective to justify a post.

### 10.2 Job-description ideation

Target job descriptions should surface potentially valuable concepts and themes rather than dictate wording.

The goal is: "these are topics, concepts, and skills that may be worth touching on," not "insert these keywords into everything."

### 10.3 Public professional profiles

The user's own LinkedIn profile can help WorkLore understand public positioning, career chronology, visible claims, and gaps between public positioning and private evidence.

Selected public profiles may help build audience context. One-person targeting is supplemental, not the central product behavior.

### 10.4 Comments and replies

Comments and replies should eventually use:

`user voice x original poster presence x subject seriousness`

A whimsical source post may support a lighter reply. A serious source may still permit measured levity or dark humor when consistent with the user's voice and the context, but not enough to trivialize the subject.

## 11. Resume ingestion policy

Resume ingestion is an optional bootstrap utility called conceptually "Seed from resume."

It is not a core product workflow.

The existing local TXT/MD/PDF/DOCX extraction and work-history candidate implementation may remain as useful prototype foundation, but robust arbitrary resume parsing is de-emphasized. Engineering effort should not chase endless formatting edge cases unless evidence shows clear product value.

A user must be able to build a complete and useful WorkLore vault without importing a resume.

## 12. Deployment and architecture boundary

WorkLore is a Windows-first standalone desktop application.

Accepted architecture:

- Tauri 2 desktop shell;
- React and TypeScript UI;
- Rust native services where appropriate;
- durable local canonical storage;
- SQLite or similar durable local memory/indexing where useful;
- portable/exportable human-readable records where practical;
- Ollama as a local model path;
- Gemini or other provider paths only through user-authorized credentials or explicit manual workspace export/import;
- web/news retrieval only for explicit research and discovery functions;
- no WorkLore-hosted backend required;
- no WorkLore account required;
- no proprietary WorkLore cloud sync in the accepted roadmap;
- no automatic social publishing or scheduling in the accepted roadmap.

The user may place a portable build or vault on a user-managed storage location such as Google Drive. Cross-machine concurrent editing and synchronization guarantees are not an early requirement.

## 13. Navigation direction

The existing resume-centric navigation should be replaced by a task-oriented model approximately centered on:

- Capture
- Stories
- Topics
- Voice
- Posts
- Insights

Sources, privacy, provider configuration, audit history, and other infrastructure remain accessible but are not the user's primary mental model.

A home surface should emphasize likely actions such as:

- Capture something
- Develop a story
- Explore post ideas
- Draft or refine a post
- Review what is working

## 14. MVP and accepted roadmap boundary

The accepted roadmap is intentionally bounded.

In scope over the planned phases:

- durable professional memory;
- story and topic capture;
- themes and proof points;
- inspiration and target-context ingestion;
- voice learning and controlled voice evolution;
- anti-slop and writing-pattern review;
- confidentiality transformation;
- evidence and standing review;
- angle generation;
- Audience Lens;
- post drafting and revision history;
- published-post archive;
- analytics import and performance learning;
- news/topic discovery;
- comments/replies as assisted drafting artifacts;
- direct local or user-authorized model providers where they support these workflows.

Explicitly excluded from the accepted roadmap:

- WorkLore-hosted SaaS backend;
- WorkLore accounts;
- proprietary cloud vault synchronization;
- multi-user collaboration;
- automatic LinkedIn or social publishing;
- LinkedIn scheduling integration;
- autonomous commenting or engagement;
- optimizing output for volume as a primary goal;
- making robust arbitrary resume parsing a major engineering program.

## 15. Success measures

WorkLore should optimize for quality and compounding usefulness rather than post volume.

Useful product measures include:

### Draft usefulness

How often a selected angle and substantive core survive into the final approved version.

### Edit distance and edit quality

Whether the amount of corrective rewriting decreases over time while intentional user edits remain meaningful.

### Voice fidelity

Whether user-approved output increasingly matches core voice and selected tone mode without collapsing into one style.

### Evidence coverage

How much substantive content can trace claims to stories, proof points, and explicit sources.

### Content diversity

Whether themes, stories, proof points, structures, tones, and angles rotate instead of repeating mechanically.

### Capture conversion

How often raw thoughts become useful stories, topics, proof points, or publishable content.

### Audience signal

Which content characteristics correlate with outcomes the user values, such as profile views, saves, relevant comments, or engagement from target readers.

### Slop prevention

How often WorkLore helps the user improve, rethink, or discard generic or unsupported content before publication.

## 16. Product-quality gates

Before a broad initial offering, WorkLore should demonstrate:

1. A user can create a useful vault without a resume.
2. Capture is durable and low-friction.
3. Stories, story seeds, proof points, topics, themes, inspiration, and target context persist locally.
4. The voice model has auditable provenance and cannot learn from raw AI drafts.
5. Tone modes provide intentional range without losing author identity.
6. Public drafts can reliably apply confidentiality transformation without destroying factual meaning.
7. Angle generation creates genuinely different approaches before prose generation.
8. Audience Lens provides useful reader-signal analysis without simply producing audience-specific rewrites.
9. Evidence/standing review can challenge unsupported thought leadership.
10. The anti-slop linter detects meaningful single-draft and cross-draft patterns.
11. Revision history preserves raw draft, user changes, approved version, and source lineage.
12. Imported analytics can be joined to published-post metadata and produce confidence-aware observations.
13. No product workflow requires a WorkLore-hosted service.
14. No product workflow automatically publishes or schedules social content.

## 17. Source guidance incorporated into this PRD

The July 2026 Career Story Bank guidance establishes several durable product ideas adopted here:

- career stories are richer source material than resume bullets;
- speaking and guided questioning are valid ways to recover detail;
- stories should preserve real private facts while discretion is applied downstream;
- one resume bullet may contain several distinct stories;
- STAR is a completeness check rather than a required visible template;
- story material should remain reusable across resumes, LinkedIn content, interviews, and other career work.

The July 2026 LinkedIn Content Engine guidance establishes several content-system ideas adopted here:

- voice identity and generation behavior should remain separate;
- configurations should be living rather than repeatedly rebuilt;
- content themes should be surfaced from documented experience rather than invented;
- angle diversity is necessary to avoid mode collapse;
- proof points should rotate rather than repeat mechanically;
- content types and tones should vary intentionally;
- analytics should inform future topics, cadence, and experiments;
- profile views and saves can be more useful signals than raw likes depending on the user's goal;
- authority should come from demonstrated experience rather than influencer framing.

Where this PRD goes beyond those guides, it does so intentionally through product decisions made for WorkLore, including local canonical storage, durable topic/inspiration/target-context objects, edit-delta voice learning, Audience Lens, anti-slop resistance, standalone deployment, and the explicit human publication boundary.
