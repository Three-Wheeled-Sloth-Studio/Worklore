# WorkLore Quick Start

WorkLore is a local-first professional-memory workspace. The fastest useful path is to capture evidence about work you have already done, turn the promising fragments into reusable Stories, connect those Stories to topics and context, then draft from that grounded material later.

## 1. Create or open a vault

A vault is the local folder that holds WorkLore's canonical data. New vaults default beside the WorkLore executable you launched, and the location can be changed before creation. WorkLore remembers the last opened vault until you explicitly close it.

## 2. Start with resume bullets as professional-memory seeds

Manual copy/paste is the recommended bootstrap path for the current QA build.

1. Open **Capture**.
2. Leave the source type as **Note or pasted text**.
3. Copy one resume bullet and paste it into the capture box.
4. Choose **Save capture**. Do not rewrite the bullet first; preserving the original wording gives WorkLore useful source lineage.
5. Classify the saved capture based on what it represents:
   - **Story seed**: a situation, project, decision, problem, or result that could become a fuller professional Story.
   - **Proof point**: compact evidence such as a metric, scale, duration, revenue result, cost reduction, cycle-time improvement, team size, or other concrete outcome.
   - A capture can support both roles when that is accurate.
6. If you linked it as a Story seed, choose **Develop this story seed**.
7. Answer the guided questions with only what you actually remember. Mark each answer as a confirmed fact, reasonable estimate, uncertain memory, or not applicable. **I do not remember** and **Skip for now** are valid answers.
8. When the guided pass is complete, choose **Create developing story**.

The original resume bullet remains part of the Story's lineage. The goal is not to turn the resume into polished prose again; it is to recover the richer memory behind the compressed bullet.

### Practical bootstrap pattern

Work through the resume one meaningful bullet at a time. Prioritize bullets that contain one or more of these signals:

- a quantified outcome;
- a product, system, or initiative you owned;
- a difficult decision or tradeoff;
- a before/after change;
- a team, stakeholder, or customer challenge;
- a result you can explain with more context than the resume allows.

Skip generic responsibility bullets until there is something specific worth preserving.

## 3. Capture new material as it happens

Use **Home -> Quick capture** for a fast note, or **Capture** for the full workflow. Save the raw memory first. Classification is optional and can happen later.

Useful captures include results, decisions, questions, ideas, excerpts, URLs, customer observations, interview notes, and remembered context from older work.

## 4. Build reusable Stories

Use **Stories** to revisit work already in motion. Story development is evidence-oriented rather than polish-oriented: preserve uncertainty instead of inventing detail, and add context only when you can support it.

A useful Story should eventually give you enough grounded material to explain what was happening, what you did, why you did it, and what changed.

## 5. Connect context before drafting

Use **Topics** to connect the material that belongs together before asking for prose. Topics can connect Stories, Proof Points, Inspiration, and Target Context while keeping those roles distinct.

Use **Voice** separately for writing evidence and governed style preferences. External Inspiration should not silently become evidence about your own work or training material for your Voice.

## 6. AI is optional

Open **AI settings** from Settings. You can use local Ollama, or bring your own OpenAI or Gemini API key. BYOK keys are protected for the current Windows user and are never returned to the UI after saving. External provider prompts pass through WorkLore privacy preflight before they leave the machine, and never-send entities remain redacted.

Professional-memory capture, classification, Story Seed development, and the manual editorial workflow remain usable with no AI provider configured. Provider use is explicit; WorkLore does not silently fall back between local and cloud providers. API usage is billed separately by the selected provider and is not included with consumer ChatGPT or Gemini subscriptions.

## 7. Draft, challenge, publish, and learn

Use **Posts** when you are ready to turn grounded material into content. The current workflow supports drafting, deterministic challenge checks, revision history, confidentiality preflight, and explicit approval of one exact public-safe revision.

Publication remains manual and outside WorkLore. After you publish, use **Insights** to record the exact publication and real performance metrics. Early feedback is treated as descriptive evidence, not as proof that one writing choice caused a result.

## A useful first session

For a 20-30 minute first pass:

1. Create/open your vault.
2. Paste three to five strong resume bullets into Capture, one at a time.
3. Mark each as Story seed, Proof point, or both.
4. Fully develop one Story seed.
5. Configure Ollama or a BYOK provider only if you want provider-assisted work; AI is not required for the memory workflow.
6. Stop there and reopen the vault later. The important first win is a small amount of trustworthy professional memory, not a large amount of generated text.
