import type { CandidateStatus, CandidateSummary } from "../domain/types";

interface StoryCandidatePanelProps {
  candidates: CandidateSummary[];
  onInterview: (candidateId: string) => Promise<void>;
  onStatusChange: (candidateId: string, status: CandidateStatus) => Promise<void>;
}

export function StoryCandidatePanel({
  candidates,
  onInterview,
  onStatusChange,
}: StoryCandidatePanelProps) {
  if (candidates.length === 0) {
    return null;
  }

  return (
    <section className="workspace-panel candidate-panel" aria-labelledby="candidate-heading">
      <div className="panel-heading-row">
        <div>
          <p className="eyebrow">Story bank</p>
          <h2 id="candidate-heading">Story candidates</h2>
          <p className="review-progress">
            {candidates.length} resume claim{candidates.length === 1 ? "" : "s"} ready to triage
          </p>
        </div>
      </div>

      <div className="candidate-list">
        {candidates.map((candidate) => (
          <article className={`story-candidate ${candidate.status}`} key={candidate.candidateId}>
            <div className="candidate-copy">
              {candidate.surroundingHeading ? (
                <p className="candidate-context">{candidate.surroundingHeading}</p>
              ) : null}
              <h3>{candidate.claim}</h3>
              <div className="candidate-signals">
                {candidate.metrics.map((metric) => (
                  <span className="status-pill" key={metric}>
                    Metric: {metric}
                  </span>
                ))}
                <span className="status-pill">
                  {candidate.missingFields.length} gap
                  {candidate.missingFields.length === 1 ? "" : "s"}
                </span>
              </div>
              {candidate.missingFields.length > 0 ? (
                <p className="candidate-gaps">
                  Needs: {candidate.missingFields.map(humanizeField).join(", ")}
                </p>
              ) : null}
            </div>

            <div className="candidate-actions" aria-label={`Actions for ${candidate.claim}`}>
              <button
                className="primary-button compact"
                disabled={candidate.status === "interviewing"}
                onClick={() => void onInterview(candidate.candidateId)}
              >
                {candidate.status === "ready_to_interview" ? "Start interview" : "Interview"}
              </button>
              <button
                className="secondary-button compact"
                disabled={candidate.status === "saved_for_later"}
                onClick={() =>
                  void onStatusChange(candidate.candidateId, "saved_for_later")
                }
              >
                Later
              </button>
              <button
                className="text-button"
                disabled={candidate.status === "ignored"}
                onClick={() => void onStatusChange(candidate.candidateId, "ignored")}
              >
                Ignore
              </button>
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}

function humanizeField(value: string): string {
  return value.replaceAll("_", " ");
}
