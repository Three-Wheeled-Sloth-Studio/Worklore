import { useEffect, useState } from "react";
import type {
  AnswerClassification,
  InterviewResponseAction,
  InterviewSummary,
} from "../domain/types";

interface GuidedInterviewPanelProps {
  interview: InterviewSummary | null;
  onSubmit: (
    interviewId: string,
    action: InterviewResponseAction,
    text: string,
    classification: AnswerClassification | null,
  ) => Promise<void>;
}

export function GuidedInterviewPanel({ interview, onSubmit }: GuidedInterviewPanelProps) {
  const [answer, setAnswer] = useState("");
  const [classification, setClassification] =
    useState<AnswerClassification>("confirmed_fact");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    setAnswer("");
    setClassification("confirmed_fact");
  }, [interview?.interviewId, interview?.currentQuestion]);

  if (!interview) {
    return null;
  }

  const complete = interview.status === "ready_for_synthesis";

  async function submit(action: InterviewResponseAction) {
    setSubmitting(true);
    try {
      await onSubmit(
        interview.interviewId,
        action,
        action === "answer" ? answer : "",
        action === "answer" ? classification : null,
      );
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <section className="workspace-panel interview-panel" aria-labelledby="interview-heading">
      <div className="interview-header">
        <div>
          <p className="eyebrow">Guided interview</p>
          <h2 id="interview-heading">
            {complete ? "Interview pass complete" : "Fill in the story behind the bullet"}
          </h2>
        </div>
        <span className="interview-progress-pill">
          {interview.completedFieldCount}/{interview.totalFieldCount} complete
        </span>
      </div>

      <div className="interview-claim">
        <span>Resume claim</span>
        <strong>{interview.candidateClaim}</strong>
      </div>

      {complete ? (
        <div className="interview-complete">
          <h3>Ready for synthesis</h3>
          <p>
            WorkLore has completed this interview pass. Partial answers and remembered gaps
            remain labeled rather than being polished into imaginary facts.
          </p>
          <button className="primary-button compact" disabled>
            Synthesize story - next slice
          </button>
        </div>
      ) : (
        <div className="interview-question-layout">
          <div className="interview-question">
            <p className="interview-field">
              {humanizeField(interview.currentTargetField ?? "story detail")}
            </p>
            <h3>{interview.currentQuestion}</h3>
          </div>

          <div className="interview-answer">
            <label htmlFor="interview-answer">Your answer</label>
            <textarea
              id="interview-answer"
              value={answer}
              onChange={(event) => setAnswer(event.target.value)}
              placeholder="Answer naturally. Rough facts beat polished fog."
              rows={6}
            />

            <div className="interview-answer-controls">
              <label>
                <span>Evidence level</span>
                <select
                  value={classification}
                  onChange={(event) =>
                    setClassification(event.target.value as AnswerClassification)
                  }
                >
                  <option value="confirmed_fact">Confirmed fact</option>
                  <option value="user_estimate">Reasonable estimate</option>
                  <option value="uncertain">Uncertain memory</option>
                  <option value="not_applicable">Not applicable</option>
                </select>
              </label>

              <div className="interview-actions">
                <button
                  className="primary-button compact"
                  disabled={submitting || answer.trim().length === 0}
                  onClick={() => void submit("answer")}
                >
                  Save and continue
                </button>
                <button
                  className="secondary-button compact"
                  disabled={submitting}
                  onClick={() => void submit("do_not_remember")}
                >
                  I do not remember
                </button>
                <button
                  className="text-button"
                  disabled={submitting}
                  onClick={() => void submit("skip")}
                >
                  Skip for now
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}

function humanizeField(value: string): string {
  return value.replaceAll("_", " ");
}
