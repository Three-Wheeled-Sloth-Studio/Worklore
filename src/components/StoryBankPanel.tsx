import { useEffect, useMemo, useState } from "react";
import type { CaptureSource, StoryStatus, StorySummary } from "../domain/types";
import { errorMessage } from "../domain/types";
import { listCaptureSources } from "../lib/captureApi";
import {
  classifyCaptureSource,
  getLastVaultPath,
  listStories,
} from "../lib/workloreApi";
import { InfoButton } from "./InfoButton";
import { SeedDevelopmentPanel } from "./SeedDevelopmentPanel";

interface StoryBankPanelProps {
  stories: StorySummary[];
  onStatusChange: (storyId: string, status: StoryStatus) => Promise<void>;
}

export function StoryBankPanel({ stories, onStatusChange }: StoryBankPanelProps) {
  const [vaultPath, setVaultPath] = useState<string | null>(null);
  const [captures, setCaptures] = useState<CaptureSource[]>([]);
  const [localStories, setLocalStories] = useState<StorySummary[]>(stories);
  const [developmentSeedId, setDevelopmentSeedId] = useState<string | null>(null);
  const [busySourceId, setBusySourceId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLocalStories(stories);
  }, [stories]);

  useEffect(() => {
    void refresh();
  }, []);

  const memorySeeds = useMemo(
    () =>
      captures.filter((capture) =>
        capture.classifications.some(
          (item) => item.role === "story_seed" || item.role === "proof_point",
        ),
      ),
    [captures],
  );

  async function refresh() {
    try {
      const path = await getLastVaultPath();
      if (!path) return;
      setVaultPath(path);
      const [captureRows, storyRows] = await Promise.all([
        listCaptureSources(path, 250),
        listStories(path),
      ]);
      setCaptures(captureRows);
      setLocalStories(storyRows);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function develop(capture: CaptureSource) {
    if (!vaultPath) return;
    setBusySourceId(capture.sourceId);
    setError(null);
    try {
      const existing = capture.classifications.find((item) => item.role === "story_seed");
      const seedId = existing
        ? existing.targetId
        : (await classifyCaptureSource(vaultPath, capture.sourceId, "story_seed")).targetId;
      await refresh();
      setDevelopmentSeedId(seedId);
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusySourceId(null);
    }
  }

  async function handleStatusChange(storyId: string, status: StoryStatus) {
    await onStatusChange(storyId, status);
    await refresh();
  }

  if (developmentSeedId && vaultPath) {
    return (
      <SeedDevelopmentPanel
        vaultPath={vaultPath}
        seedId={developmentSeedId}
        onClose={() => {
          setDevelopmentSeedId(null);
          void refresh();
        }}
      />
    );
  }

  if (memorySeeds.length === 0 && localStories.length === 0) {
    return null;
  }

  return (
    <section className="workspace-panel story-bank-panel" aria-labelledby="story-bank-heading">
      <div className="compact-section-heading">
        <h2 id="story-bank-heading">Stories</h2>
        <InfoButton label="Story Seed and Proof Point guidance">
          Story Seed means there is a situation, decision, challenge, or result worth unpacking into a
          reusable story. Proof Point means there is a concrete fact, metric, scale, or outcome worth
          citing. A strong resume bullet is often both. Proof-only items remain reusable evidence; use
          the develop action when there is a story behind one.
        </InfoButton>
      </div>

      {memorySeeds.length > 0 ? (
        <div className="story-seed-section">
          <div className="compact-subheading">
            <h3>Memory seeds</h3>
            <span className="status-pill">{memorySeeds.length}</span>
          </div>
          <div className="story-seed-list">
            {memorySeeds.map((capture) => {
              const storySeed = capture.classifications.some((item) => item.role === "story_seed");
              const proofPoint = capture.classifications.some((item) => item.role === "proof_point");
              return (
                <article className="story-seed-row" key={capture.sourceId}>
                  <div className="story-seed-copy">
                    <strong>{capture.displayName}</strong>
                    <span>
                      {storySeed ? "Story seed" : ""}
                      {storySeed && proofPoint ? " · " : ""}
                      {proofPoint ? "Proof point" : ""}
                    </span>
                  </div>
                  <button
                    className="shell-icon-button primary-icon"
                    type="button"
                    disabled={busySourceId === capture.sourceId}
                    aria-label={`Develop ${capture.displayName}`}
                    title={storySeed ? "Develop story" : "Develop as story; also marks this as a Story Seed"}
                    onClick={() => void develop(capture)}
                  >
                    <DevelopIcon />
                  </button>
                </article>
              );
            })}
          </div>
        </div>
      ) : null}

      {localStories.length > 0 ? (
        <div className="story-bank-section">
          <div className="compact-subheading">
            <h3>Developed</h3>
            <span className="status-pill">{localStories.length}</span>
          </div>
          <div className="story-bank-list">
            {localStories.map((story) => (
              <article className={`story-card ${story.status}`} key={story.storyId}>
                <div className="story-card-heading">
                  <div>
                    <h3>{story.title}</h3>
                    <p className="story-role">
                      {[story.organizationName, story.roleTitle].filter(Boolean).join(" | ") ||
                        humanize(story.storyType)}
                    </p>
                  </div>
                  <div className="story-card-badges">
                    {story.metrics.slice(0, 3).map((metric) => (
                      <span className="status-pill" key={metric}>{metric}</span>
                    ))}
                    <span className="status-pill">{humanize(story.status)}</span>
                  </div>
                </div>

                {story.summary ? <p className="story-summary">{story.summary}</p> : null}

                <div className="story-card-footer">
                  <div className="story-actions">
                    {story.status === "ready_for_review" ? (
                      <button
                        className="secondary-button compact"
                        onClick={() => void handleStatusChange(story.storyId, "validated")}
                      >
                        Validate
                      </button>
                    ) : null}
                    {story.status === "validated" ? (
                      <button
                        className="primary-button compact"
                        onClick={() => void handleStatusChange(story.storyId, "finalized")}
                      >
                        Finalize
                      </button>
                    ) : null}
                    {story.status !== "archived" ? (
                      <button
                        className="text-button"
                        onClick={() => void handleStatusChange(story.storyId, "archived")}
                      >
                        Archive
                      </button>
                    ) : null}
                  </div>
                </div>
              </article>
            ))}
          </div>
        </div>
      ) : null}

      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </section>
  );
}

function DevelopIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M5 19l4.5-1 9-9-3.5-3.5-9 9L5 19zM13.5 7l3.5 3.5M5 19l4-4" />
    </svg>
  );
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}
