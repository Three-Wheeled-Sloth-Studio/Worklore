import { useEffect, useState } from "react";
import type { TopicRecord } from "../domain/types";
import { errorMessage } from "../domain/types";
import { createTopic, listTopics } from "../lib/workloreApi";
import { InfoButton } from "./InfoButton";
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
    if (!newTitle.trim()) return;
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
    <section className="workspace-panel compact-workspace-panel" aria-labelledby="topics-workspace-heading">
      <div className="compact-section-heading">
        <h2 id="topics-workspace-heading">Topics</h2>
        <InfoButton label="About Topics">
          Capture an idea, optionally connect Story or Proof Point standing and other context, then
          generate the first Post draft from the Topic.
        </InfoButton>
      </div>
      <div className="inline-create-row">
        <input
          value={newTitle}
          onChange={(event) => setNewTitle(event.target.value)}
          placeholder="New topic"
          aria-label="New topic title"
          onKeyDown={(event) => {
            if (event.key === "Enter") void createNewTopic();
          }}
        />
        <button
          className="shell-icon-button primary-icon"
          type="button"
          disabled={busy || !newTitle.trim()}
          aria-label="Add Topic"
          title="Add Topic"
          onClick={() => void createNewTopic()}
        >
          <PlusIcon />
        </button>
      </div>
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
      {topics.length === 0 ? (
        <div className="compact-empty-state">No Topics yet.</div>
      ) : (
        <div className="record-list">
          {topics.map((topic) => (
            <button className="record-row" key={topic.topicId} onClick={() => setSelectedTopicId(topic.topicId)}>
              <span>
                <strong>{topic.title}</strong>
                {topic.summary ? <small>{topic.summary}</small> : null}
              </span>
              <span className="record-meta">{topic.lifecycle} · {topic.relationships.length}</span>
            </button>
          ))}
        </div>
      )}
    </section>
  );
}

function PlusIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>;
}
