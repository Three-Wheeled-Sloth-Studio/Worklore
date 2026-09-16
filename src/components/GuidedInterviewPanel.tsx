import { useEffect, useState } from "react";
import type {
  AnswerClassification,
  InterviewResponseAction,
  InterviewSummary,
  ManualWorkspaceTarget,
} from "../domain/types";

interface GuidedInterviewPanelProps {
  interview: InterviewSummary | null;
  onSubmit: (
    interviewId: string,
    action: InterviewResponseAction,
    text: string,
    classification: AnswerClassification | null,
  ) => Promise<void>;
  onExportWorkspace: (
    interviewId: string,
    target: ManualWorkspaceTarget,
  ) => Promise<void>;
  onImportResponse: (interviewId: string) => Promise<void>;
}

export function GuidedInterviewPanel({
  interview,
  onSubmit,
  onExportWorkspace,
  onImportResponse,
}: GuidedInterviewPanelProps) {
  const [answer, setAnswer] = useState("");
  const [classification, setClassification] =
    useState<AnswerClassification>("confirmed_fact");
  const [workspaceTarget, setWorkspaceTarget] =
    useState<ManualWorkspaceTarget>("gemini");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    setAnswer("");
    setClassification("confirmed_fact");
  }, [interview?.interviewId, interview?.currentQuestion]);

  if (!interview) {
    return null;
  }

  const activeInterview = interview;
  const complete =
    activeInterview.status === "ready_for_synthesis" || activeInterview.status === "completed";

  async function submit(action: InterviewResponseAction) {
    setSubmitting(true);
    try {
      await onSubmit(
        activeInterview.interviewId,
        action,
        action === "answer" ? answer : "",
        action === "answer" ? classification : null,
      );
    } finally {
      setSubmitting(false);
    }
  }

  async function exportWorkspace() {
    setSubmitting(true);
    try {
      await onExportWorkspace(activeInterview.interviewId, workspaceTarget);
    } finally {
      setSubmitting(false);
    }
  }

  async function importResponse() {
    setSubmitting(true);
    try {
      await onImportResponse(activeInterview.interviewId);
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
          {activeInterview.completedFieldCount}/{activeInterview.totalFieldCount} complete
        </span>
      </div>

      <div className="interview-claim">
        <span>Resume claim</span>
        <strong>{activeInterview.candidateClaim}</strong>
      </div>

      {complete ? (
        <div className="interview-complete">
          <h3>{activeInterview.status === "completed" ? "Story imported" : "Ready for synthesis"}</h3>
          <p>
            WorkLore keeps partial answers and remembered gaps labeled. Export a structured AI
            workspace, then import the JSON response after the model completes the synthesis.
          </p>
          <div className="manual-workspace-controls">
            <label>
              <span>AI workspace</span>
              <select
                value={workspaceTarget}
                onChange={(event) =>
                  setWorkspaceTarget(event.target.value as ManualWorkspaceTarget)
                }
              >
                <option value="gemini">Gemini</option>
                <option value="chatgpt">ChatGPT</option>
                <option value="claude">Claude</option>
                <option value="generic">Generic package</option>
              </select>
            </label>
            <button
              className="secondary-button compact"
              disabled={submitting}
              onClick={() => void exportWorkspace()}
            >
              Export workspace
            </button>
            <button
              className="primary-button compact"
              disabled={submitting}
              onClick={() => void importResponse()}
            >
              Import story JSON
            </button>
          </div>
          <p className="quiet-copy">
            Imported responses are schema-validated, linked to the employment role, scanned for
            private entities, and saved as paired Markdown and JSON story records.
          </p>
        </div>
      ) : (
        <div className="interview-question-layout">
          <div className="interview-question">
            <p className="interview-field">
              {humanizeField(activeInterview.currentTargetField ?? "story detail")}
            </p>
            <h3>{activeInterview.currentQuestion}</h3>
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
