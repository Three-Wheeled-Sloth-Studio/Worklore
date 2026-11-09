import {
  useEffect,
  useMemo,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import type { FeedbackSnapshotView, PerformanceMetrics } from "../domain/feedback";
import type { PostRecordView } from "../domain/posts";
import { errorMessage } from "../domain/types";
import {
  getFeedbackSnapshot,
  markPostPublished,
  recordPostPerformance,
} from "../lib/feedbackApi";
import { listPosts } from "../lib/postApi";
import "../insights.css";

const EMPTY_METRICS: PerformanceMetrics = {
  impressions: 0,
  reactions: 0,
  comments: 0,
  reposts: 0,
  saves: 0,
  profileViews: 0,
};

export function InsightsWorkspace({ vaultPath }: { vaultPath: string }) {
  const [snapshot, setSnapshot] = useState<FeedbackSnapshotView | null>(null);
  const [approvedPosts, setApprovedPosts] = useState<PostRecordView[]>([]);
  const [selectedApprovedPostId, setSelectedApprovedPostId] = useState("");
  const [publishedAt, setPublishedAt] = useState(defaultPublicationTime());
  const [publicationUrl, setPublicationUrl] = useState("");
  const [selectedPublicationId, setSelectedPublicationId] = useState("");
  const [metrics, setMetrics] = useState<Record<keyof PerformanceMetrics, string>>(
    metricsToForm(EMPTY_METRICS),
  );
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectedApprovedPost = useMemo(
    () => approvedPosts.find((post) => post.postId === selectedApprovedPostId) ?? null,
    [approvedPosts, selectedApprovedPostId],
  );
  const selectedPublication = useMemo(
    () => snapshot?.publications.find((item) => item.publicationId === selectedPublicationId) ?? null,
    [snapshot, selectedPublicationId],
  );
  const selectedPerformance = useMemo(
    () =>
      snapshot?.latestPerformance.find((item) => item.publicationId === selectedPublicationId) ??
      null,
    [snapshot, selectedPublicationId],
  );
  const alreadyPublishedPostIds = useMemo(
    () => new Set(snapshot?.publications.map((publication) => publication.postId) ?? []),
    [snapshot],
  );
  const unpublishedApprovedPosts = useMemo(
    () => approvedPosts.filter((post) => !alreadyPublishedPostIds.has(post.postId)),
    [approvedPosts, alreadyPublishedPostIds],
  );

  useEffect(() => {
    setSnapshot(null);
    setApprovedPosts([]);
    setSelectedApprovedPostId("");
    setPublishedAt(defaultPublicationTime());
    setPublicationUrl("");
    setSelectedPublicationId("");
    setMetrics(metricsToForm(EMPTY_METRICS));
    setNotes("");
    setNotice(null);
    setError(null);
    void refresh(vaultPath);
  }, [vaultPath]);

  useEffect(() => {
    if (selectedPerformance) {
      setMetrics(metricsToForm(selectedPerformance.metrics));
      setNotes(selectedPerformance.notes);
    } else {
      setMetrics(metricsToForm(EMPTY_METRICS));
      setNotes("");
    }
  }, [selectedPublicationId, selectedPerformance?.performanceId]);

  useEffect(() => {
    if (
      selectedApprovedPostId &&
      unpublishedApprovedPosts.some((post) => post.postId === selectedApprovedPostId)
    ) {
      return;
    }
    setSelectedApprovedPostId(unpublishedApprovedPosts[0]?.postId ?? "");
  }, [unpublishedApprovedPosts, selectedApprovedPostId]);

  async function refresh(path: string, preferredPublicationId?: string) {
    setBusy(true);
    setError(null);
    try {
      const [next, postResult] = await Promise.all([
        getFeedbackSnapshot(path),
        listPosts(path),
      ]);
      setSnapshot(next);
      setApprovedPosts(postResult.filter((post) => post.status === "final_approved"));
      const nextSelected =
        preferredPublicationId &&
        next.publications.some((item) => item.publicationId === preferredPublicationId)
          ? preferredPublicationId
          : next.publications[0]?.publicationId ?? "";
      setSelectedPublicationId(nextSelected);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function recordPublication() {
    if (
      !selectedApprovedPost ||
      !selectedApprovedPost.finalApprovedRevisionId ||
      !publishedAt
    ) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const publication = await markPostPublished(vaultPath, {
        postId: selectedApprovedPost.postId,
        revisionId: selectedApprovedPost.finalApprovedRevisionId,
        platform: "linkedin",
        publishedAt: new Date(publishedAt).toISOString(),
        publicationUrl: publicationUrl.trim() || null,
      });
      setPublicationUrl("");
      setPublishedAt(defaultPublicationTime());
      await refresh(vaultPath, publication.publicationId);
      setNotice(
        "Manual LinkedIn publication recorded against the exact final-approved Revision. WorkLore did not publish or schedule it.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function savePerformance() {
    if (!selectedPublicationId) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await recordPostPerformance(vaultPath, {
        publicationId: selectedPublicationId,
        metrics: formToMetrics(metrics),
        notes,
      });
      await refresh(vaultPath, selectedPublicationId);
      setNotice(
        "Performance snapshot saved. Insights remain descriptive until enough published Posts exist for a pattern check.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="insights-workspace" aria-labelledby="insights-heading">
      <div className="workspace-panel insights-intro">
        <p className="eyebrow">Phase 4 thin feedback seam</p>
        <h2 id="insights-heading">Insights</h2>
        <p>
          Record the LinkedIn publication you performed manually, then attach LinkedIn-native
          outcomes to that exact approved Revision. WorkLore reports descriptive evidence and sample
          limits instead of turning one successful Post into a rule.
        </p>
      </div>

      <div className="insights-layout">
        <div className="shell-stack">
          <section className="workspace-panel">
            <p className="eyebrow">Manual publication boundary</p>
            <h3>Record a published Post</h3>
            {unpublishedApprovedPosts.length === 0 ? (
              <div className="empty-state compact-empty">
                <p>
                  No unrecorded final-approved Posts are available. Approve a Post first, publish it
                  manually outside WorkLore, then return here.
                </p>
              </div>
            ) : (
              <>
                <label className="field-label" htmlFor="published-post">Final-approved Post</label>
                <select
                  id="published-post"
                  value={selectedApprovedPostId}
                  onChange={(event) => setSelectedApprovedPostId(event.target.value)}
                  disabled={busy}
                >
                  {unpublishedApprovedPosts.map((post) => (
                    <option key={post.postId} value={post.postId}>{post.title}</option>
                  ))}
                </select>
                <label className="field-label" htmlFor="published-at">Published at</label>
                <input
                  id="published-at"
                  type="datetime-local"
                  value={publishedAt}
                  onChange={(event) => setPublishedAt(event.target.value)}
                  disabled={busy}
                />
                <label className="field-label" htmlFor="publication-url">LinkedIn URL (optional)</label>
                <input
                  id="publication-url"
                  type="url"
                  value={publicationUrl}
                  onChange={(event) => setPublicationUrl(event.target.value)}
                  placeholder="https://www.linkedin.com/posts/..."
                  disabled={busy}
                />
                <p className="record-meta">
                  This records what you already published. It does not send anything to LinkedIn.
                </p>
                <button
                  className="primary-button compact"
                  disabled={busy || !selectedApprovedPost || !publishedAt}
                  onClick={() => void recordPublication()}
                >
                  Record manual publication
                </button>
              </>
            )}
          </section>

          <section className="workspace-panel">
            <p className="eyebrow">Published corpus</p>
            <h3>Manual publications</h3>
            {!snapshot || snapshot.publications.length === 0 ? (
              <div className="empty-state">
                <h4>No published Posts recorded</h4>
                <p>Record the external publication above after you have published an approved Post.</p>
              </div>
            ) : (
              <div className="record-list">
                {snapshot.publications.map((publication) => {
                  const measured = snapshot.latestPerformance.some(
                    (item) => item.publicationId === publication.publicationId,
                  );
                  return (
                    <button
                      key={publication.publicationId}
                      className={`record-row ${selectedPublicationId === publication.publicationId ? "selected" : ""}`}
                      onClick={() => setSelectedPublicationId(publication.publicationId)}
                    >
                      <span>
                        <strong>{publication.postTitle}</strong>
                        <small>{publication.platform} · {formatDate(publication.publishedAt)}</small>
                      </span>
                      <span className="record-meta">{measured ? "measured" : "needs metrics"}</span>
                    </button>
                  );
                })}
              </div>
            )}
          </section>
        </div>

        <div className="shell-stack">
          <section className="workspace-panel">
            <p className="eyebrow">Measure</p>
            <h3>{selectedPublication ? selectedPublication.postTitle : "Select a publication"}</h3>
            {selectedPublication ? (
              <>
                <p>
                  Enter the current cumulative values from LinkedIn. Saving again creates another
                  append-only snapshot; the latest one drives the current descriptive view.
                </p>
                <div className="metrics-grid">
                  <MetricField label="Impressions" field="impressions" metrics={metrics} setMetrics={setMetrics} />
                  <MetricField label="Reactions" field="reactions" metrics={metrics} setMetrics={setMetrics} />
                  <MetricField label="Comments" field="comments" metrics={metrics} setMetrics={setMetrics} />
                  <MetricField label="Reposts" field="reposts" metrics={metrics} setMetrics={setMetrics} />
                  <MetricField label="Saves" field="saves" metrics={metrics} setMetrics={setMetrics} />
                  <MetricField label="Profile views" field="profileViews" metrics={metrics} setMetrics={setMetrics} />
                </div>
                <label className="field-label" htmlFor="performance-notes">Notes</label>
                <textarea
                  id="performance-notes"
                  value={notes}
                  onChange={(event) => setNotes(event.target.value)}
                  placeholder="Optional context: timing, format, unusual distribution, or anything worth remembering."
                />
                <button className="primary-button compact" disabled={busy} onClick={() => void savePerformance()}>
                  Save performance snapshot
                </button>
                {selectedPerformance ? (
                  <p className="record-meta">Latest snapshot: {formatDate(selectedPerformance.recordedAt)}</p>
                ) : null}
              </>
            ) : (
              <div className="empty-state compact-empty">
                <p>No publication selected.</p>
              </div>
            )}
          </section>

          <section className="workspace-panel">
            <p className="eyebrow">Learn without overfitting</p>
            <h3>Current observations</h3>
            {!snapshot || snapshot.insights.length === 0 ? (
              <div className="empty-state compact-empty">
                <p>No observation is available yet.</p>
              </div>
            ) : (
              <div className="insight-list">
                {snapshot.insights.map((insight, index) => (
                  <article className="next-step-card" key={`${insight.kind}-${index}`}>
                    <strong>{humanize(insight.kind)}</strong>
                    <p>{insight.statement}</p>
                    <small>Measured sample: {insight.sampleSize}</small>
                  </article>
                ))}
              </div>
            )}
          </section>
        </div>
      </div>

      {notice ? <p className="inline-notice" role="status">{notice}</p> : null}
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </section>
  );
}

function MetricField({
  label,
  field,
  metrics,
  setMetrics,
}: {
  label: string;
  field: keyof PerformanceMetrics;
  metrics: Record<keyof PerformanceMetrics, string>;
  setMetrics: Dispatch<SetStateAction<Record<keyof PerformanceMetrics, string>>>;
}) {
  return (
    <label className="metric-field">
      <span>{label}</span>
      <input
        type="number"
        min="0"
        step="1"
        inputMode="numeric"
        value={metrics[field]}
        onChange={(event) =>
          setMetrics((current) => ({ ...current, [field]: event.target.value }))
        }
      />
    </label>
  );
}

function metricsToForm(metrics: PerformanceMetrics): Record<keyof PerformanceMetrics, string> {
  return {
    impressions: String(metrics.impressions),
    reactions: String(metrics.reactions),
    comments: String(metrics.comments),
    reposts: String(metrics.reposts),
    saves: String(metrics.saves),
    profileViews: String(metrics.profileViews),
  };
}

function formToMetrics(
  form: Record<keyof PerformanceMetrics, string>,
): PerformanceMetrics {
  return {
    impressions: metricNumber(form.impressions),
    reactions: metricNumber(form.reactions),
    comments: metricNumber(form.comments),
    reposts: metricNumber(form.reposts),
    saves: metricNumber(form.saves),
    profileViews: metricNumber(form.profileViews),
  };
}

function metricNumber(value: string): number {
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : 0;
}

function defaultPublicationTime(): string {
  const now = new Date();
  const local = new Date(now.getTime() - now.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 16);
}

function formatDate(value: string): string {
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) {
    return value;
  }
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(parsed);
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}
