import { useEffect, useMemo, useState } from "react";
import type { FeedbackSnapshotView, PerformanceMetrics } from "../domain/feedback";
import { errorMessage } from "../domain/types";
import { getFeedbackSnapshot, recordPostPerformance } from "../lib/feedbackApi";
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
  const [selectedPublicationId, setSelectedPublicationId] = useState("");
  const [metrics, setMetrics] = useState<Record<keyof PerformanceMetrics, string>>(
    metricsToForm(EMPTY_METRICS),
  );
  const [notes, setNotes] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

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

  useEffect(() => {
    setSnapshot(null);
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

  async function refresh(path: string, preferredPublicationId?: string) {
    setBusy(true);
    setError(null);
    try {
      const next = await getFeedbackSnapshot(path);
      setSnapshot(next);
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
          Record LinkedIn-native outcomes against the exact Revision you manually published. WorkLore
          reports descriptive evidence and sample limits instead of turning one successful Post into a rule.
        </p>
      </div>

      <div className="insights-layout">
        <section className="workspace-panel">
          <p className="eyebrow">Published corpus</p>
          <h3>Manual publications</h3>
          {!snapshot || snapshot.publications.length === 0 ? (
            <div className="empty-state">
              <h4>No published Posts recorded</h4>
              <p>Approve a Post, publish it manually, then mark that exact Revision published in Posts.</p>
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
  setMetrics: React.Dispatch<React.SetStateAction<Record<keyof PerformanceMetrics, string>>>;
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
