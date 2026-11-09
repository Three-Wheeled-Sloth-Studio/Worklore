import { useEffect, useState } from "react";
import type {
  AnswerClassification,
  InterviewResponseAction,
  StorySeedDevelopmentSummary,
} from "../domain/types";
import { errorMessage } from "../domain/types";
import {
  createStoryFromSeedDevelopment,
  startStorySeedDevelopment,
  submitStorySeedDevelopmentResponse,
} from "../lib/workloreApi";
import "../seed-development.css";

interface SeedDevelopmentPanelProps {
  vaultPath: string;
  seedId: string;
  onClose: () => void;
}

export function SeedDevelopmentPanel({
  vaultPath,
  seedId,
  onClose,
}: SeedDevelopmentPanelProps) {
  const [development, setDevelopment] = useState<StorySeedDevelopmentSummary | null>(null);
  const [answer, setAnswer] = useState("");
  const [classification, setClassification] =
    useState<AnswerClassification>("confirmed_fact");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setBusy(true);
    setError(null);
    void startStorySeedDevelopment(vaultPath, seedId)
      .then((result) => {
        if (!cancelled) {
          setDevelopment(result);
        }
      })
      .catch((caught) => {
        if (!cancelled) {
          setError(errorMessage(caught));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setBusy(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [vaultPath, seedId]);

  useEffect(() => {
    setAnswer("");
    setClassification("confirmed_fact");
  }, [development?.interviewId, development?.currentQuestion]);

  async function submit(action: InterviewResponseAction) {
    if (!development) {
      return;
    }
    setBusy(true);
    setNotice(null);
    setError(null);
    try {
      const updated = await submitStorySeedDevelopmentResponse(vaultPath, {
        interviewId: development.interviewId,
        action,
        text: action === "answer" ? answer : "",
        classification: action === "answer" ? classification : null,
      });
      setDevelopment(updated);
      setNotice(
        updated.status === "ready_for_synthesis"
          ? "Guided development pass complete. You can now create the developing Story."
          : "Saved locally. Here is the next useful question.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function createStory() {
    if (!development) {
      return;
    }
    setBusy(true);
    setNotice(null);
    setError(null);
    try {
      const result = await createStoryFromSeedDevelopment(
        vaultPath,
        development.interviewId,
      );
      setDevelopment(result.development);
      setNotice(
        result.created
          ? `Developing Story created as ${result.storyId}.`
          : `This seed is already linked to ${result.storyId}.`,
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  if (!development) {
    return (
      <div className="seed-development-card">
        <div className="seed-development-header">
          <div>
            <p className="eyebrow">Develop story</p>
            <h3>Opening Story Seed...</h3>
          </div>
          <button className="quiet-button compact" onClick={onClose}>Close</button>
        </div>
        {error ? <p className="capture-error">{error}</p> : null}
      </div>
    );
  }

  const ready = development.status === "ready_for_synthesis";
  const completed = development.status === "completed";

  return (
    <div className="seed-development-card" aria-live="polite">
      <div className="seed-development-header">
        <div>
          <p className="eyebrow">Develop story</p>
          <h3>{development.seedTitle}</h3>
          <p>{development.seedSummary}</p>
        </div>
        <button className="quiet-button compact" onClick={onClose}>Close</button>
      </div>

      <div className="seed-development-progress">
        <span>{development.completedFieldCount}/{development.totalFieldCount} evidence-rich fields</span>
        <span>{development.interviewId}</span>
      </div>

      {completed ? (
        <div className="seed-development-ready">
          <strong>Developing Story created</strong>
          <p>{development.storyId}</p>
          <p className="quiet-copy">
            The Story keeps explicit lineage to this Story Seed and its classified interview answers.
          </p>
        </div>
      ) : ready ? (
        <div className="seed-development-ready">
          <strong>Guided pass complete</strong>
          <p>
            Create a local developing Story from this seed and the attributed answers. No provider call is required.
          </p>
          <button
            className="primary-button compact"
            disabled={busy}
            onClick={() => void createStory()}
          >
            Create developing story
          </button>
        </div>
      ) : (
        <div className="seed-development-question">
          <span>{humanizeField(development.currentTargetField ?? "story detail")}</span>
          <h4>{development.currentQuestion}</h4>
          <textarea
            aria-label="Story development answer"
            value={answer}
            onChange={(event) => setAnswer(event.target.value)}
            placeholder="Answer naturally. Preserve uncertainty rather than polishing over it."
            rows={5}
          />
          <div className="seed-development-controls">
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
            <div className="seed-development-actions">
              <button
                className="primary-button compact"
                disabled={busy || answer.trim().length === 0}
                onClick={() => void submit("answer")}
              >
                Save and continue
              </button>
              <button
                className="secondary-button compact"
                disabled={busy}
                onClick={() => void submit("do_not_remember")}
              >
                I do not remember
              </button>
              <button
                className="text-button"
                disabled={busy}
                onClick={() => void submit("skip")}
              >
                Skip for now
              </button>
            </div>
          </div>
        </div>
      )}

      <p className="seed-development-boundary">
        This direct Story Seed workflow stays local. The legacy manual-AI export is not reused until its privacy preflight is generalized for canonical seed/story targets.
      </p>
      {busy ? <p>Saving...</p> : null}
      {notice ? <p className="capture-notice">{notice}</p> : null}
      {error ? <p className="capture-error">{error}</p> : null}
    </div>
  );
}

function humanizeField(value: string): string {
  return value.replaceAll("_", " ");
}
