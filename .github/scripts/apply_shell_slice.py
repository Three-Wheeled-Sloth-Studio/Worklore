from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


write(
    "src/navigation.ts",
    '''export type PrimaryView = "home" | "capture" | "stories" | "topics" | "voice" | "posts" | "insights";
export type LibraryView = "sources" | "privacy" | "import_export";
export type AppView = PrimaryView | LibraryView | "settings";
export type WorkspaceAvailability = "available" | "planned";

export interface NavigationItem {
  id: AppView;
  label: string;
  availability: WorkspaceAvailability;
  description: string;
}

export const PRIMARY_NAV_ITEMS: NavigationItem[] = [
  { id: "home", label: "Home", availability: "available", description: "Orient and continue work" },
  { id: "capture", label: "Capture", availability: "available", description: "Save first, classify second" },
  { id: "stories", label: "Stories", availability: "available", description: "Develop professional memory" },
  { id: "topics", label: "Topics", availability: "available", description: "Connect ideas before drafting" },
  { id: "voice", label: "Voice", availability: "planned", description: "Phase 2 voice intelligence" },
  { id: "posts", label: "Posts", availability: "planned", description: "Phase 3 editorial workflow" },
  { id: "insights", label: "Insights", availability: "planned", description: "Phase 4 learning loop" },
];

export const LIBRARY_NAV_ITEMS: NavigationItem[] = [
  { id: "sources", label: "Sources", availability: "available", description: "Provenance, Inspiration, and Target Context" },
  { id: "privacy", label: "Privacy", availability: "available", description: "Private entities and public-use controls" },
  { id: "import_export", label: "Import / Export", availability: "available", description: "Supporting transfer workflows" },
];

export const SETTINGS_NAV_ITEM: NavigationItem = {
  id: "settings",
  label: "Settings",
  availability: "available",
  description: "Providers, storage, and diagnostics",
};

export function isPlannedPrimaryView(view: AppView): view is "voice" | "posts" | "insights" {
  return view === "voice" || view === "posts" || view === "insights";
}
''',
)

write(
    "src/navigation.test.ts",
    '''import { describe, expect, it } from "vitest";
import { LIBRARY_NAV_ITEMS, PRIMARY_NAV_ITEMS, SETTINGS_NAV_ITEM } from "./navigation";

describe("task-oriented navigation contract", () => {
  it("keeps the accepted primary workspaces in order", () => {
    expect(PRIMARY_NAV_ITEMS.map((item) => item.id)).toEqual([
      "home",
      "capture",
      "stories",
      "topics",
      "voice",
      "posts",
      "insights",
    ]);
  });

  it("marks current Phase 1 workspaces available and future phases planned", () => {
    expect(
      PRIMARY_NAV_ITEMS.filter((item) => item.availability === "available").map((item) => item.id),
    ).toEqual(["home", "capture", "stories", "topics"]);
    expect(
      PRIMARY_NAV_ITEMS.filter((item) => item.availability === "planned").map((item) => item.id),
    ).toEqual(["voice", "posts", "insights"]);
  });

  it("keeps infrastructure out of primary navigation", () => {
    expect(LIBRARY_NAV_ITEMS.map((item) => item.id)).toEqual([
      "sources",
      "privacy",
      "import_export",
    ]);
    expect(SETTINGS_NAV_ITEM.id).toBe("settings");
    expect(PRIMARY_NAV_ITEMS.some((item) => item.label.toLowerCase().includes("resume"))).toBe(false);
  });
});
''',
)

write(
    "src/components/HomeWorkspace.tsx",
    '''import { useState } from "react";
import type { InterviewSummary } from "../domain/types";
import { errorMessage } from "../domain/types";
import { createCaptureSource } from "../lib/workloreApi";
import type { AppView } from "../navigation";

export function HomeWorkspace({
  vaultPath,
  storyCount,
  candidateCount,
  activeInterview,
  privacyReviewCount,
  onNavigate,
  onCaptureSaved,
}: {
  vaultPath: string;
  storyCount: number;
  candidateCount: number;
  activeInterview: InterviewSummary | null;
  privacyReviewCount: number;
  onNavigate: (view: AppView) => void;
  onCaptureSaved: () => Promise<void> | void;
}) {
  const [text, setText] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function saveQuickCapture() {
    if (!text.trim()) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await createCaptureSource(vaultPath, text, "other");
      setText("");
      setNotice("Saved locally as a neutral Source. Classify it later when the destination is clear.");
      await onCaptureSaved();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="shell-stack">
      <section className="workspace-panel home-hero" aria-labelledby="home-heading">
        <p className="eyebrow">Professional memory</p>
        <h2 id="home-heading">What do you want to move forward?</h2>
        <p>
          Capture something while it is fresh, develop a Story, or connect an idea to the context
          that makes it useful. WorkLore stays useful without an AI provider configured.
        </p>
        <div className="quick-action-grid">
          <button className="primary-button" onClick={() => onNavigate("capture")}>Capture something</button>
          <button className="secondary-button" onClick={() => onNavigate("stories")}>Develop a story</button>
          <button className="secondary-button" onClick={() => onNavigate("topics")}>Explore topics</button>
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="quick-capture-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Quick capture</p>
            <h2 id="quick-capture-heading">Save first. Sort it out later.</h2>
          </div>
        </div>
        <textarea
          className="quick-capture-input"
          value={text}
          onChange={(event) => setText(event.target.value)}
          placeholder="A result you remembered, a question, an idea, a URL, an excerpt..."
          rows={4}
        />
        <div className="primary-actions">
          <button className="primary-button compact" disabled={busy || !text.trim()} onClick={() => void saveQuickCapture()}>
            {busy ? "Saving..." : "Save locally"}
          </button>
          <button className="quiet-button compact" onClick={() => onNavigate("capture")}>Open full Capture</button>
        </div>
        {notice ? <p className="inline-notice">{notice}</p> : null}
        {error ? <p className="inline-error" role="alert">{error}</p> : null}
      </section>

      <section className="workspace-panel" aria-labelledby="continue-heading">
        <p className="eyebrow">Continue</p>
        <h2 id="continue-heading">Work already in motion</h2>
        <div className="continue-grid">
          {activeInterview ? (
            <button className="continue-card" onClick={() => onNavigate("stories")}>
              <strong>Continue story interview</strong>
              <span>{activeInterview.currentQuestion ?? "The interview is ready for its next step."}</span>
            </button>
          ) : null}
          {candidateCount > 0 ? (
            <button className="continue-card" onClick={() => onNavigate("stories")}>
              <strong>{candidateCount} story seed{candidateCount === 1 ? "" : "s"} from resume bootstrap</strong>
              <span>Optional imported career material is waiting for review.</span>
            </button>
          ) : null}
          {privacyReviewCount > 0 ? (
            <button className="continue-card" onClick={() => onNavigate("privacy")}>
              <strong>{privacyReviewCount} privacy review{privacyReviewCount === 1 ? "" : "s"}</strong>
              <span>Resolve private-entity ambiguity before public or provider use.</span>
            </button>
          ) : null}
          {!activeInterview && candidateCount === 0 && privacyReviewCount === 0 ? (
            <div className="empty-state compact-empty">
              <h3>No urgent follow-up</h3>
              <p>Capture something new or explore the professional memory you already have.</p>
            </div>
          ) : null}
        </div>
      </section>

      <section className="workspace-panel" aria-labelledby="explore-heading">
        <p className="eyebrow">Explore</p>
        <h2 id="explore-heading">Build connections before prose</h2>
        <div className="quick-action-grid">
          <button className="continue-card" onClick={() => onNavigate("topics")}>
            <strong>Topics</strong>
            <span>Connect Stories, Proof Points, Inspiration, and Target Context.</span>
          </button>
          <button className="continue-card" onClick={() => onNavigate("sources")}>
            <strong>Sources and context</strong>
            <span>Reopen provenance, Inspiration, and opportunity context.</span>
          </button>
          <button className="continue-card" onClick={() => onNavigate("stories")}>
            <strong>{storyCount} canonical Stor{storyCount === 1 ? "y" : "ies"}</strong>
            <span>Review or continue developing reusable professional memory.</span>
          </button>
        </div>
      </section>
    </div>
  );
}
''',
)

write(
    "src/components/TopicsWorkspace.tsx",
    '''import { useEffect, useState } from "react";
import type { TopicRecord } from "../domain/types";
import { errorMessage } from "../domain/types";
import { createTopic, listTopics } from "../lib/workloreApi";
import { TopicPanel } from "./TopicPanel";

export function TopicsWorkspace({ vaultPath }: { vaultPath: string }) {
  const [topics, setTopics] = useState<TopicRecord[]>([]);
  const [selectedTopicId, setSelectedTopicId] = useState<string | null>(null);
  const [newTitle, setNewTitle] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedTopicId(null);
    void refresh();
  }, [vaultPath]);

  async function refresh() {
    try {
      setTopics(await listTopics(vaultPath));
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function createNewTopic() {
    if (!newTitle.trim()) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      const topic = await createTopic(vaultPath, {
        title: newTitle.trim(),
        summary: "",
        timingClass: "evergreen",
      });
      setNewTitle("");
      setSelectedTopicId(topic.topicId);
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  if (selectedTopicId) {
    return (
      <TopicPanel
        vaultPath={vaultPath}
        topicId={selectedTopicId}
        onClose={() => {
          setSelectedTopicId(null);
          void refresh();
        }}
      />
    );
  }

  return (
    <section className="workspace-panel" aria-labelledby="topics-workspace-heading">
      <div className="panel-heading-row">
        <div>
          <p className="eyebrow">Connect before drafting</p>
          <h2 id="topics-workspace-heading">Topics</h2>
          <p>Durable ideas can stay useful even when they never become a Post.</p>
        </div>
      </div>
      <div className="inline-create-row">
        <input
          value={newTitle}
          onChange={(event) => setNewTitle(event.target.value)}
          placeholder="New topic"
          aria-label="New topic title"
        />
        <button className="primary-button compact" disabled={busy || !newTitle.trim()} onClick={() => void createNewTopic()}>
          Add topic
        </button>
      </div>
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
      {topics.length === 0 ? (
        <div className="empty-state">
          <h3>No topics yet</h3>
          <p>Capture an idea as a Topic Candidate or add one here, then connect standing and context.</p>
        </div>
      ) : (
        <div className="record-list">
          {topics.map((topic) => (
            <button className="record-row" key={topic.topicId} onClick={() => setSelectedTopicId(topic.topicId)}>
              <span>
                <strong>{topic.title}</strong>
                <small>{topic.summary || "No summary yet."}</small>
              </span>
              <span className="record-meta">{topic.lifecycle} · {topic.timingClass} · {topic.relationships.length} links</span>
            </button>
          ))}
        </div>
      )}
    </section>
  );
}
''',
)

write(
    "src/components/LibraryWorkspace.tsx",
    '''import { useEffect, useState } from "react";
import type { InspirationRecord, SourceSummary, SourceType, TargetContextRecord } from "../domain/types";
import { errorMessage } from "../domain/types";
import { listInspirations, listTargetContexts } from "../lib/workloreApi";
import { InspirationPanel } from "./InspirationPanel";
import { TargetContextPanel } from "./TargetContextPanel";

export function LibraryWorkspace({
  vaultPath,
  sources,
  sourceTypeOptions,
  selectedSourceType,
  onSourceTypeChange,
  onImportSource,
  onExtractCandidates,
}: {
  vaultPath: string;
  sources: SourceSummary[];
  sourceTypeOptions: Array<{ value: SourceType; label: string }>;
  selectedSourceType: SourceType;
  onSourceTypeChange: (sourceType: SourceType) => void;
  onImportSource: () => void;
  onExtractCandidates: (sourceId: string) => void;
}) {
  const [inspirations, setInspirations] = useState<InspirationRecord[]>([]);
  const [targets, setTargets] = useState<TargetContextRecord[]>([]);
  const [selectedInspirationId, setSelectedInspirationId] = useState<string | null>(null);
  const [selectedTargetId, setSelectedTargetId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedInspirationId(null);
    setSelectedTargetId(null);
    void refreshContext();
  }, [vaultPath]);

  async function refreshContext() {
    try {
      const [inspirationRows, targetRows] = await Promise.all([
        listInspirations(vaultPath),
        listTargetContexts(vaultPath),
      ]);
      setInspirations(inspirationRows);
      setTargets(targetRows);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  if (selectedInspirationId) {
    return (
      <InspirationPanel
        vaultPath={vaultPath}
        inspirationId={selectedInspirationId}
        onClose={() => {
          setSelectedInspirationId(null);
          void refreshContext();
        }}
      />
    );
  }

  if (selectedTargetId) {
    return (
      <TargetContextPanel
        vaultPath={vaultPath}
        targetId={selectedTargetId}
        onClose={() => {
          setSelectedTargetId(null);
          void refreshContext();
        }}
      />
    );
  }

  return (
    <div className="shell-stack">
      <section className="workspace-panel source-panel" aria-labelledby="library-sources-heading">
        <div className="panel-heading-row">
          <div>
            <p className="eyebrow">Provenance library</p>
            <h2 id="library-sources-heading">Sources</h2>
            <p>Original material stays neutral. Semantic roles are explicit working records.</p>
          </div>
          <div className="import-controls">
            <select
              aria-label="Source type"
              value={selectedSourceType}
              onChange={(event) => onSourceTypeChange(event.target.value as SourceType)}
            >
              {sourceTypeOptions.map((option) => (
                <option key={option.value} value={option.value}>{option.label}</option>
              ))}
            </select>
            <button className="primary-button compact" onClick={onImportSource}>Import file</button>
          </div>
        </div>
        {sources.length === 0 ? (
          <div className="empty-state">
            <h3>No imported files yet</h3>
            <p>Capture pasted material directly, or import a supporting document when file provenance matters.</p>
          </div>
        ) : (
          <div className="source-list">
            {sources.map((source) => (
              <article className="source-row" key={source.sourceId}>
                <div>
                  <h3>{source.displayName}</h3>
                  <p>{sourceTypeOptions.find((item) => item.value === source.sourceType)?.label ?? "Source"} · Imported {formatDate(source.importedAt)}</p>
                </div>
                <div className="source-statuses">
                  <span className="status-pill">Text: {source.extractionStatus}</span>
                  <span className={`status-pill ${source.privacyScanStatus === "needs_review" ? "attention" : ""}`}>Privacy: {source.privacyScanStatus}</span>
                  {source.sourceType === "resume" && source.extractionStatus === "complete" ? (
                    <button className="quiet-button compact" onClick={() => onExtractCandidates(source.sourceId)}>Seed from resume</button>
                  ) : null}
                </div>
              </article>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="inspiration-library-heading">
        <p className="eyebrow">External creative context</p>
        <h2 id="inspiration-library-heading">Inspiration</h2>
        <p>External material can influence thinking without becoming Evidence or Voice Evidence.</p>
        {inspirations.length === 0 ? (
          <div className="empty-state compact-empty"><p>No Inspiration records yet. Capture a URL, excerpt, or source and classify it explicitly.</p></div>
        ) : (
          <div className="record-list">
            {inspirations.map((item) => (
              <button className="record-row" key={item.inspirationId} onClick={() => setSelectedInspirationId(item.inspirationId)}>
                <span><strong>{item.title}</strong><small>{item.summary || item.whyInteresting || "No summary yet."}</small></span>
                <span className="record-meta">{item.lifecycle} · {item.relationships.length} links</span>
              </button>
            ))}
          </div>
        )}
      </section>

      <section className="workspace-panel" aria-labelledby="target-library-heading">
        <p className="eyebrow">External professional context</p>
        <h2 id="target-library-heading">Target Context</h2>
        <p>Requirements and audience signals describe the opportunity. They do not prove user standing.</p>
        {targets.length === 0 ? (
          <div className="empty-state compact-empty"><p>No Target Context records yet. Capture a job description or other target material and classify it explicitly.</p></div>
        ) : (
          <div className="record-list">
            {targets.map((item) => (
              <button className="record-row" key={item.targetId} onClick={() => setSelectedTargetId(item.targetId)}>
                <span><strong>{item.title}</strong><small>{item.summary || item.organizationName || "No summary yet."}</small></span>
                <span className="record-meta">{item.lifecycle} · {item.relationships.length} links</span>
              </button>
            ))}
          </div>
        )}
      </section>
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </div>
  );
}

function formatDate(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", { dateStyle: "medium" }).format(parsed);
}
''',
)

write(
    "src/components/FutureWorkspace.tsx",
    '''export function FutureWorkspace({ view }: { view: "voice" | "posts" | "insights" }) {
  const copy = {
    voice: {
      eyebrow: "Phase 2",
      title: "Voice",
      body: "Core Voice, Tone Modes, Writing Rules, Voice Direction, and provenance-governed Voice Evidence are intentionally not implemented yet.",
      boundary: "Raw AI drafts will never become canonical Voice Evidence.",
    },
    posts: {
      eyebrow: "Phase 3",
      title: "Posts",
      body: "Angle development, drafting, challenge, editorial review, and manual-publication tracking are planned after Voice provenance is stable.",
      boundary: "WorkLore will not auto-publish or schedule social content.",
    },
    insights: {
      eyebrow: "Phase 4",
      title: "Insights",
      body: "Analytics import, experiments, repetition checks, and confidence-aware learning are planned after the editorial workflow exists.",
      boundary: "No engagement-maximization score or fake KPI is being shown before evidence exists.",
    },
  }[view];

  return (
    <section className="workspace-panel future-workspace" aria-labelledby={`${view}-heading`}>
      <p className="eyebrow">{copy.eyebrow}</p>
      <h2 id={`${view}-heading`}>{copy.title}</h2>
      <p>{copy.body}</p>
      <div className="next-step-card">
        <h3>Not a placeholder pretending to work</h3>
        <p>{copy.boundary}</p>
      </div>
    </section>
  );
}
''',
)

write(
    "src/shell.css",
    '''.product-shell {
  min-height: calc(100vh - 44px);
  display: grid;
  grid-template-columns: minmax(190px, 232px) minmax(0, 1fr);
  background: var(--page-bg, #f4f1ea);
}

.shell-sidebar {
  position: sticky;
  top: 0;
  align-self: start;
  min-height: 100vh;
  padding: 24px 16px;
  border-right: 1px solid rgba(36, 42, 48, 0.12);
  background: rgba(255, 255, 255, 0.78);
  backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  gap: 22px;
}

.shell-brand h1 { margin: 2px 0 4px; font-size: 1.55rem; }
.shell-brand p { margin: 0; }
.shell-vault-name { color: #5a646c; font-size: 0.86rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.shell-nav-section { display: grid; gap: 6px; }
.shell-nav-label { margin: 0 8px 3px; font-size: 0.72rem; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: #707980; }
.shell-nav-button {
  width: 100%;
  border: 0;
  border-radius: 10px;
  padding: 9px 10px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.shell-nav-button:hover { background: rgba(67, 79, 89, 0.08); }
.shell-nav-button.active { background: rgba(67, 79, 89, 0.14); font-weight: 700; }
.shell-nav-button .soon-badge { font-size: 0.68rem; font-weight: 700; text-transform: uppercase; color: #777; }
.shell-settings-link { margin-top: auto; }

.shell-content { min-width: 0; padding: 28px clamp(18px, 4vw, 54px) 48px; }
.shell-topbar { display: flex; align-items: flex-start; justify-content: space-between; gap: 18px; margin-bottom: 18px; }
.shell-topbar h2 { margin: 2px 0 4px; font-size: clamp(1.4rem, 2.5vw, 2rem); }
.shell-topbar .vault-path { max-width: 72ch; }
.shell-page { max-width: 1180px; }
.shell-stack { display: grid; gap: 18px; }
.shell-workspace-grid { align-items: start; }
.quick-action-grid, .continue-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(190px, 1fr)); gap: 12px; }
.quick-capture-input { width: 100%; resize: vertical; margin-bottom: 10px; }
.continue-card, .record-row {
  border: 1px solid rgba(40, 50, 58, 0.14);
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.72);
  padding: 14px;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.continue-card { display: grid; gap: 5px; }
.continue-card span, .record-row small { color: #606a72; line-height: 1.45; }
.record-list { display: grid; gap: 8px; }
.record-row { width: 100%; display: flex; justify-content: space-between; align-items: center; gap: 18px; }
.record-row > span:first-child { min-width: 0; display: grid; gap: 4px; }
.record-row small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.record-meta { flex: 0 0 auto; color: #68737b; font-size: 0.78rem; text-transform: capitalize; }
.inline-create-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: 10px; margin: 14px 0; }
.inline-notice { color: #335f48; }
.inline-error { color: #8a332e; }
.compact-empty { padding: 12px 0; }
.home-hero { padding-block: 28px; }
.future-workspace { max-width: 760px; }
.support-actions { display: flex; flex-wrap: wrap; gap: 10px; }

@media (max-width: 840px) {
  .product-shell { grid-template-columns: 1fr; }
  .shell-sidebar { position: static; min-height: auto; border-right: 0; border-bottom: 1px solid rgba(36, 42, 48, 0.12); }
  .shell-nav-section { grid-template-columns: repeat(auto-fit, minmax(110px, 1fr)); }
  .shell-nav-label { grid-column: 1 / -1; }
  .shell-settings-link { margin-top: 0; }
  .shell-topbar { flex-direction: column; }
  .record-row { align-items: flex-start; flex-direction: column; gap: 7px; }
}
''',
)

app = read("src/App.tsx")
app = replace_once(
    app,
    'import { CapturePanel } from "./components/CapturePanel";\n',
    'import { CapturePanel } from "./components/CapturePanel";\nimport { FutureWorkspace } from "./components/FutureWorkspace";\nimport { HomeWorkspace } from "./components/HomeWorkspace";\nimport { LibraryWorkspace } from "./components/LibraryWorkspace";\n',
    "shell component imports",
)
app = replace_once(
    app,
    'import { StoryCandidatePanel } from "./components/StoryCandidatePanel";\n',
    'import { StoryCandidatePanel } from "./components/StoryCandidatePanel";\nimport { TopicsWorkspace } from "./components/TopicsWorkspace";\n',
    "topics import",
)
app = replace_once(
    app,
    'import "./styles.css";\n',
    'import type { AppView } from "./navigation";\nimport { LIBRARY_NAV_ITEMS, PRIMARY_NAV_ITEMS, SETTINGS_NAV_ITEM } from "./navigation";\nimport "./styles.css";\nimport "./shell.css";\n',
    "navigation imports",
)
app = replace_once(app, 'const [vaultName, setVaultName] = useState("My Career Stories");', 'const [vaultName, setVaultName] = useState("My WorkLore");', "default vault name")
app = replace_once(app, 'vaultName.trim() || "My Career Stories",', 'vaultName.trim() || "My WorkLore",', "create vault fallback")
app = replace_once(
    app,
    '  const [error, setError] = useState<string | null>(null);\n',
    '  const [error, setError] = useState<string | null>(null);\n  const [activeView, setActiveView] = useState<AppView>("home");\n',
    "active view state",
)
app = replace_once(
    app,
    '            Create a local career story bank or open an existing vault. New vaults are stored in\n            WorkLore\'s application folder and reopen automatically until you explicitly close\n            them.\n',
    '            Create a local professional-memory vault or open an existing one. New vaults are\n            stored in WorkLore\'s application folder and reopen automatically until you explicitly\n            close them.\n',
    "landing copy",
)

render_marker = '  if (!startupComplete) {\n'
render_workspace = '''  function renderWorkspace() {
    switch (activeView) {
      case "home":
        return (
          <HomeWorkspace
            vaultPath={vault!.path}
            storyCount={stories.length}
            candidateCount={candidates.filter((item) => item.status !== "ignored" && item.status !== "converted_to_story").length}
            activeInterview={activeInterview}
            privacyReviewCount={reviews.length}
            onNavigate={setActiveView}
            onCaptureSaved={() => refreshWorkspace(vault!.path, true)}
          />
        );
      case "capture":
        return <CapturePanel vaultPath={vault!.path} />;
      case "stories":
        return (
          <div className="workspace-grid shell-workspace-grid">
            <GuidedInterviewPanel
              interview={activeInterview}
              onSubmit={handleInterviewResponse}
              onExportWorkspace={handleExportWorkspace}
              onImportResponse={handleImportStoryResponse}
            />
            <StoryBankPanel stories={stories} onStatusChange={handleStoryStatusChange} />
            <StoryCandidatePanel
              candidates={candidates.filter(
                (candidate) =>
                  candidate.status !== "ignored" &&
                  candidate.status !== "interviewing" &&
                  candidate.status !== "converted_to_story",
              )}
              onInterview={handleStartInterview}
              onStatusChange={handleCandidateStatusChange}
            />
          </div>
        );
      case "topics":
        return <TopicsWorkspace vaultPath={vault!.path} />;
      case "voice":
      case "posts":
      case "insights":
        return <FutureWorkspace view={activeView} />;
      case "sources":
        return (
          <LibraryWorkspace
            vaultPath={vault!.path}
            sources={sources}
            sourceTypeOptions={SOURCE_TYPES}
            selectedSourceType={selectedSourceType}
            onSourceTypeChange={setSelectedSourceType}
            onImportSource={() => void handleImportSource()}
            onExtractCandidates={(sourceId) => void handleExtractCandidates(sourceId)}
          />
        );
      case "privacy":
        return (
          <div className="shell-stack">
            {reviews.length > 0 ? (
              <PrivacyReviewPanel reviews={reviews} onResolve={handleResolveReview} />
            ) : (
              <section className="workspace-panel">
                <p className="eyebrow">Private Entity Registry</p>
                <h2>Privacy</h2>
                <div className="empty-state compact-empty">
                  <h3>No unresolved entity reviews</h3>
                  <p>Stable private tokens and prior decisions remain in the local vault.</p>
                </div>
              </section>
            )}
            <section className="workspace-panel settings-panel" aria-labelledby="privacy-mode-heading">
              <p className="eyebrow">Public and provider boundary</p>
              <h2 id="privacy-mode-heading">Private names</h2>
              <p>Choose the default behavior when content leaves the local-only boundary. Every supported export still requires privacy preflight.</p>
              <div className="segmented-control" role="group" aria-label="Cloud private name mode">
                <button className={vault!.cloudIdentifierMode === "redact" ? "active" : ""} onClick={() => void handlePrivacyModeChange("redact")}>Use stable tokens</button>
                <button className={vault!.cloudIdentifierMode === "include" ? "active" : ""} onClick={() => void handlePrivacyModeChange("include")}>Include names</button>
              </div>
            </section>
          </div>
        );
      case "import_export":
        return (
          <section className="workspace-panel" aria-labelledby="import-export-heading">
            <p className="eyebrow">Supporting workflows</p>
            <h2 id="import-export-heading">Import / Export</h2>
            <p>File ingestion is available through Sources. Privacy-safe manual AI workspace export/import remains attached to Story interviews where its provenance is clear.</p>
            <div className="support-actions">
              <button className="primary-button compact" onClick={() => setActiveView("sources")}>Open Sources</button>
              <button className="secondary-button compact" onClick={() => setActiveView("stories")}>Open Stories</button>
            </div>
            <div className="next-step-card">
              <h3>Portable vault export</h3>
              <p>A general human-readable vault snapshot is part of the storage contract but is not implemented in this Phase 1 shell slice.</p>
            </div>
          </section>
        );
      case "settings":
        return (
          <div className="shell-stack">
            <section className="workspace-panel" aria-labelledby="settings-heading">
              <p className="eyebrow">Application</p>
              <h2 id="settings-heading">Settings</h2>
              <div className="next-step-card">
                <h3>AI Providers</h3>
                <p>Ollama and provider-neutral BYOK configuration begin in Phase 2. No provider is required for current professional-memory workflows, and WorkLore will not silently fall back to cloud execution.</p>
              </div>
              <div className="next-step-card">
                <h3>Storage</h3>
                <p title={vault!.path}>This vault is local at {vault!.path}. Backup and portable export remain explicit user-controlled operations.</p>
              </div>
            </section>
            <PerformancePanel snapshot={performance} onRefresh={() => refreshPerformance(vault!.path)} />
          </div>
        );
    }
  }

'''
app = replace_once(app, render_marker, render_workspace + render_marker, "workspace renderer")

start_marker = '  return (\n    <>\n      <main className="app-shell">'
end_marker = '\n  );\n}\n\nfunction StatusItem'
if start_marker not in app or end_marker not in app:
    raise RuntimeError("logged-in shell markers not found")
prefix, remainder = app.split(start_marker, 1)
_, suffix = remainder.split(end_marker, 1)
new_return = '''  return (
    <>
      <main className="product-shell">
        <aside className="shell-sidebar" aria-label="WorkLore navigation">
          <div className="shell-brand">
            <p className="eyebrow">Three-Wheeled Sloth Studio</p>
            <h1>WorkLore</h1>
            <p className="shell-vault-name" title={vault.path}>{vault.name}</p>
          </div>

          <nav className="shell-nav-section" aria-label="Primary workspaces">
            <p className="shell-nav-label">Work</p>
            {PRIMARY_NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                className={`shell-nav-button ${activeView === item.id ? "active" : ""}`}
                aria-current={activeView === item.id ? "page" : undefined}
                title={item.description}
                onClick={() => setActiveView(item.id)}
              >
                <span>{item.label}</span>
                {item.availability === "planned" ? <span className="soon-badge">Soon</span> : null}
              </button>
            ))}
          </nav>

          <nav className="shell-nav-section" aria-label="Library and supporting tools">
            <p className="shell-nav-label">Library</p>
            {LIBRARY_NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                className={`shell-nav-button ${activeView === item.id ? "active" : ""}`}
                aria-current={activeView === item.id ? "page" : undefined}
                title={item.description}
                onClick={() => setActiveView(item.id)}
              >
                <span>{item.label}</span>
              </button>
            ))}
          </nav>

          <button
            className={`shell-nav-button shell-settings-link ${activeView === SETTINGS_NAV_ITEM.id ? "active" : ""}`}
            aria-current={activeView === SETTINGS_NAV_ITEM.id ? "page" : undefined}
            onClick={() => setActiveView(SETTINGS_NAV_ITEM.id)}
          >
            <span>{SETTINGS_NAV_ITEM.label}</span>
          </button>
        </aside>

        <section className="shell-content">
          <header className="shell-topbar">
            <div>
              <p className="eyebrow">Local vault</p>
              <h2>{PRIMARY_NAV_ITEMS.concat(LIBRARY_NAV_ITEMS, SETTINGS_NAV_ITEM).find((item) => item.id === activeView)?.label ?? "WorkLore"}</h2>
              <p className="vault-path" title={vault.path}>{vault.path}</p>
            </div>
            <div className="header-actions">
              <button className="quiet-button" onClick={() => void handleOpenVault()}>Open another vault</button>
              <button className="quiet-button" onClick={() => void handleCloseVault()}>Close vault</button>
            </div>
          </header>

          <section className="status-strip" aria-label="Vault status">
            <StatusItem value={sources.length} label="Sources" />
            <StatusItem value={stories.length} label="Stories" />
            <StatusItem value={reviews.length} label="Privacy reviews" attention={reviews.length > 0} />
          </section>

          <div className="shell-page">{renderWorkspace()}</div>
          <Feedback notice={notice} error={error} />
          <BusyLayer message={busyMessage} activeOperation={activeOperation} />
        </section>
      </main>
      <footer className="legal-notice">
        <span>WorkLore is licensed under AGPL-3.0-only.</span>
        <a href="https://github.com/Three-Wheeled-Sloth-Studio/Worklore/blob/main/LICENSE" target="_blank" rel="noreferrer">View license</a>
      </footer>
    </>
  );
}

function StatusItem'''
app = prefix + new_return + suffix
write("src/App.tsx", app)
