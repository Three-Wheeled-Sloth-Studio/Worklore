import type { OperationMetric, PerformanceSnapshot } from "../domain/types";

interface PerformancePanelProps {
  snapshot: PerformanceSnapshot | null;
  onRefresh: () => Promise<void>;
}

export function PerformancePanel({ snapshot, onRefresh }: PerformancePanelProps) {
  if (!snapshot) {
    return null;
  }

  const slowestSteps = snapshot.recentMetrics
    .filter((metric) => metric.parentRunId !== null)
    .sort((left, right) => right.durationMs - left.durationMs)
    .slice(0, 8);
  const recentRuns = snapshot.recentMetrics
    .filter((metric) => metric.parentRunId === null)
    .slice(0, 6);

  return (
    <section className="workspace-panel performance-panel" aria-labelledby="performance-heading">
      <div className="panel-heading-row">
        <div>
          <p className="eyebrow">Local instrumentation</p>
          <h2 id="performance-heading">Operation timing</h2>
          <p className="performance-intro">
            WorkLore records durations and phases locally. It does not record resume text,
            interview answers, or provider payloads in these metrics.
          </p>
        </div>
        <button className="quiet-button compact" onClick={() => void onRefresh()}>
          Refresh timings
        </button>
      </div>

      {snapshot.activeOperations.length > 0 ? (
        <div className="active-operation-list" aria-label="Active operations">
          {snapshot.activeOperations.map((operation) => (
            <article className="active-operation" key={operation.runId}>
              <span className="spinner small" aria-hidden="true" />
              <div>
                <strong>{humanize(operation.operation)}</strong>
                <p>
                  {humanize(operation.phase)} | {formatDuration(operation.elapsedMs)}
                  {progressLabel(operation.progressCurrent, operation.progressTotal)}
                </p>
              </div>
            </article>
          ))}
        </div>
      ) : null}

      <div className="performance-grid">
        <div>
          <h3>Slowest recent phases</h3>
          {slowestSteps.length === 0 ? (
            <p className="quiet-copy">Run an import to begin collecting timing data.</p>
          ) : (
            <MetricTable metrics={slowestSteps} showOperation />
          )}
        </div>
        <div>
          <h3>Recent operations</h3>
          {recentRuns.length === 0 ? (
            <p className="quiet-copy">No completed operations yet.</p>
          ) : (
            <MetricTable metrics={recentRuns} />
          )}
        </div>
      </div>
    </section>
  );
}

function MetricTable({
  metrics,
  showOperation = false,
}: {
  metrics: OperationMetric[];
  showOperation?: boolean;
}) {
  return (
    <div className="metric-table" role="table">
      {metrics.map((metric) => (
        <div className="metric-row" role="row" key={metric.runId}>
          <div role="cell">
            <strong>{humanize(showOperation ? metric.phase : metric.operation)}</strong>
            {showOperation ? <span>{humanize(metric.operation)}</span> : null}
          </div>
          <span className={`metric-outcome ${metric.outcome}`} role="cell">
            {metric.outcome}
          </span>
          <span role="cell">{formatDuration(metric.durationMs)}</span>
        </div>
      ))}
    </div>
  );
}

function progressLabel(current: number | null, total: number | null): string {
  if (current === null) {
    return "";
  }
  return total === null ? ` | ${current}` : ` | ${current}/${total}`;
}

function formatDuration(durationMs: number): string {
  if (durationMs < 1000) {
    return `${durationMs} ms`;
  }
  const seconds = durationMs / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(seconds < 10 ? 1 : 0)} sec`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  return `${minutes} min ${remainingSeconds} sec`;
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}
