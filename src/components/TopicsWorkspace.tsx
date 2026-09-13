import { useEffect, useState } from "react";
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
