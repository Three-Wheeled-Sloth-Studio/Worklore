import { useEffect, useMemo, useState } from "react";
import type { ConfidentialityTransformResult } from "../domain/confidentiality";
import type { PostLineageView, PostRecordView } from "../domain/posts";
import type { LintDraftResult, StorySummary, TopicRecord } from "../domain/types";
import { errorMessage } from "../domain/types";
import { transformConfidentialityForPublicUse } from "../lib/confidentialityApi";
import {
  appendPostRevision,
  approvePostRevision,
  createPost,
  getPostLineage,
  linkPostSupportingMaterial,
  listPosts,
} from "../lib/postApi";
import { lintDraft, listStories, listTopics } from "../lib/workloreApi";
import "../posts.css";

interface ChallengeResult {
  text: string;
  lint: LintDraftResult;
  confidentiality: ConfidentialityTransformResult;
}

export function PostWorkspace({ vaultPath }: { vaultPath: string }) {
  const [posts, setPosts] = useState<PostRecordView[]>([]);
  const [topics, setTopics] = useState<TopicRecord[]>([]);
  const [stories, setStories] = useState<StorySummary[]>([]);
  const [selectedPostId, setSelectedPostId] = useState<string | null>(null);
  const [lineage, setLineage] = useState<PostLineageView | null>(null);
  const [editorText, setEditorText] = useState("");
  const [newTitle, setNewTitle] = useState("");
  const [newText, setNewText] = useState("");
  const [newTopicId, setNewTopicId] = useState("");
  const [newStoryId, setNewStoryId] = useState("");
  const [challenge, setChallenge] = useState<ChallengeResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const currentRevision = useMemo(() => {
    if (!lineage) {
      return null;
    }
    return (
      lineage.revisions.find(
        (revision) => revision.revisionId === lineage.post.currentRevisionId,
      ) ?? null
    );
  }, [lineage]);

  const unsavedChanges = Boolean(currentRevision && editorText !== currentRevision.text);
  const challengeMatchesEditor = challenge?.text === editorText;
  const publicSafeMatchesEditor = challenge?.confidentiality.publicSafeText === editorText;
  const standingLinks =
    lineage?.supportingMaterial.filter((item) =>
      ["evidence_source", "evidence", "story", "proof_point"].includes(item.role),
    ) ?? [];
  const canApprove = Boolean(
    lineage?.post.status === "working" &&
      currentRevision &&
      !unsavedChanges &&
      challengeMatchesEditor &&
      challenge?.confidentiality.state === "ready" &&
      publicSafeMatchesEditor,
  );

  useEffect(() => {
    setSelectedPostId(null);
    setLineage(null);
    setEditorText("");
    setChallenge(null);
    setNotice(null);
    setError(null);
    void loadWorkspace(vaultPath);
  }, [vaultPath]);

  async function loadWorkspace(path: string, preferredPostId?: string | null) {
    setBusy(true);
    setError(null);
    try {
      const [postResult, topicResult, storyResult] = await Promise.all([
        listPosts(path),
        listTopics(path),
        listStories(path),
      ]);
      setPosts(postResult);
      setTopics(topicResult);
      setStories(storyResult);
      const nextId =
        preferredPostId && postResult.some((post) => post.postId === preferredPostId)
          ? preferredPostId
          : postResult[0]?.postId ?? null;
      if (nextId) {
        await openPost(path, nextId);
      } else {
        setSelectedPostId(null);
        setLineage(null);
        setEditorText("");
        setChallenge(null);
      }
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function refreshCatalog(path: string, preferredPostId: string) {
    const postResult = await listPosts(path);
    setPosts(postResult);
    if (postResult.some((post) => post.postId === preferredPostId)) {
      setSelectedPostId(preferredPostId);
    }
  }

  async function openPost(path: string, postId: string) {
    setError(null);
    try {
      const loaded = await getPostLineage(path, postId);
      const current = loaded.revisions.find(
        (revision) => revision.revisionId === loaded.post.currentRevisionId,
      );
      setSelectedPostId(postId);
      setLineage(loaded);
      setEditorText(current?.text ?? "");
      setChallenge(null);
      setNotice(null);
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function createDraft() {
    if (!newText.trim()) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      let created = await createPost(vaultPath, {
        title: newTitle.trim() || "Untitled post",
        text: newText,
        origin: "user",
        authorshipState: "user_authored",
        providerRunId: null,
        providerId: null,
        modelId: null,
      });
      if (newTopicId) {
        created = await linkPostSupportingMaterial(vaultPath, {
          postId: created.post.postId,
          role: "topic",
          targetId: newTopicId,
        });
      }
      if (newStoryId) {
        created = await linkPostSupportingMaterial(vaultPath, {
          postId: created.post.postId,
          role: "story",
          targetId: newStoryId,
        });
      }
      setNewTitle("");
      setNewText("");
      setNewTopicId("");
      setNewStoryId("");
      setLineage(created);
      setSelectedPostId(created.post.postId);
      setEditorText(created.revisions.at(-1)?.text ?? "");
      setChallenge(null);
      await refreshCatalog(vaultPath, created.post.postId);
      setNotice("Draft saved with immutable revision provenance.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function saveRevision() {
    if (!lineage || !currentRevision || !unsavedChanges) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const authorshipState =
        currentRevision.authorshipState === "model_generated" ||
        currentRevision.authorshipState === "user_edited_model"
          ? "user_edited_model"
          : "user_authored";
      const updated = await appendPostRevision(vaultPath, {
        postId: lineage.post.postId,
        text: editorText,
        origin: "user",
        authorshipState,
        providerRunId: null,
        providerId: null,
        modelId: null,
      });
      setLineage(updated);
      setChallenge(null);
      await refreshCatalog(vaultPath, updated.post.postId);
      setNotice("Revision saved. Challenge the exact saved text before final approval.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function challengeDraft() {
    if (!editorText.trim()) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const [lint, confidentiality] = await Promise.all([
        lintDraft(vaultPath, { text: editorText }),
        transformConfidentialityForPublicUse(vaultPath, { text: editorText }),
      ]);
      setChallenge({ text: editorText, lint, confidentiality });
      if (confidentiality.state !== "ready") {
        setNotice(
          "Challenge found unresolved confidentiality work. Final approval remains blocked.",
        );
      } else if (confidentiality.publicSafeText !== editorText) {
        setNotice(
          "Challenge produced different public-safe text. Load it, save a new Revision, and challenge that exact Revision before approval.",
        );
      } else {
        setNotice("Challenge complete. Review advisory findings before approval.");
      }
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function approveFinal() {
    if (!lineage || !currentRevision || !canApprove) {
      return;
    }
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const approved = await approvePostRevision(vaultPath, {
        postId: lineage.post.postId,
        revisionId: currentRevision.revisionId,
      });
      setLineage(approved);
      await refreshCatalog(vaultPath, approved.post.postId);
      setNotice(
        "Final text frozen. Publish it manually outside WorkLore, then record that publication in Insights.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  function loadPublicSafeText() {
    if (!challenge || challenge.confidentiality.state !== "ready") {
      return;
    }
    setEditorText(challenge.confidentiality.publicSafeText);
    setChallenge(null);
    setNotice(
      "Public-safe text loaded as an unsaved edit. Save it as a new Revision, then challenge it again.",
    );
  }

  async function copyApprovedText() {
    if (!currentRevision || lineage?.post.status !== "final_approved") {
      return;
    }
    try {
      await navigator.clipboard.writeText(currentRevision.text);
      setNotice(
        "Approved text copied. Publish it manually, then record the publication in Insights.",
      );
    } catch {
      setNotice("Clipboard access was unavailable. Select the approved text and copy it manually.");
    }
  }

  return (
    <section className="posts-workspace" aria-labelledby="posts-heading">
      <div className="workspace-panel posts-intro">
        <p className="eyebrow">Phase 3 vertical loop</p>
        <h2 id="posts-heading">Posts</h2>
        <p>
          Draft, preserve revisions, challenge the exact text, and explicitly approve a final
          version. WorkLore does not publish or schedule social content.
        </p>
      </div>

      <div className="posts-layout">
        <div className="posts-sidebar shell-stack">
          <section className="workspace-panel">
            <div className="panel-heading-row">
              <div>
                <p className="eyebrow">Start from what you know</p>
                <h3>New draft</h3>
              </div>
            </div>
            <label className="field-label" htmlFor="post-title">Title</label>
            <input
              id="post-title"
              value={newTitle}
              onChange={(event) => setNewTitle(event.target.value)}
              placeholder="Working title"
              disabled={busy}
            />
            <label className="field-label" htmlFor="post-topic">Topic context</label>
            <select
              id="post-topic"
              value={newTopicId}
              onChange={(event) => setNewTopicId(event.target.value)}
              disabled={busy}
            >
              <option value="">No topic linked</option>
              {topics.map((topic) => (
                <option key={topic.topicId} value={topic.topicId}>{topic.title}</option>
              ))}
            </select>
            <label className="field-label" htmlFor="post-story">Standing / story</label>
            <select
              id="post-story"
              value={newStoryId}
              onChange={(event) => setNewStoryId(event.target.value)}
              disabled={busy}
            >
              <option value="">No story linked</option>
              {stories.map((story) => (
                <option key={story.storyId} value={story.storyId}>{story.title}</option>
              ))}
            </select>
            <label className="field-label" htmlFor="post-draft">Draft text</label>
            <textarea
              id="post-draft"
              className="post-draft-input"
              value={newText}
              onChange={(event) => setNewText(event.target.value)}
              placeholder="Write the first version here. Angle generation comes in a later Content Studio slice."
              disabled={busy}
            />
            <button
              className="primary-button compact"
              disabled={busy || !newText.trim()}
              onClick={() => void createDraft()}
            >
              Save draft
            </button>
          </section>

          <section className="workspace-panel">
            <p className="eyebrow">Durable lineage</p>
            <h3>Saved posts</h3>
            {posts.length === 0 ? (
              <div className="empty-state compact-empty">
                <p>No Posts yet.</p>
              </div>
            ) : (
              <div className="record-list post-record-list">
                {posts.map((post) => (
                  <button
                    key={post.postId}
                    className={`record-row ${selectedPostId === post.postId ? "selected" : ""}`}
                    onClick={() => void openPost(vaultPath, post.postId)}
                  >
                    <span>
                      <strong>{post.title}</strong>
                      <small>{post.status === "final_approved" ? "Final approved" : "Working"}</small>
                    </span>
                    <span className="record-meta">r{post.revision}</span>
                  </button>
                ))}
              </div>
            )}
          </section>
        </div>

        <div className="posts-main shell-stack">
          {!lineage || !currentRevision ? (
            <section className="workspace-panel empty-state">
              <h3>Select or create a Post</h3>
              <p>The editor uses canonical Post/Revision lineage, not temporary browser state.</p>
            </section>
          ) : (
            <>
              <section className="workspace-panel">
                <div className="panel-heading-row">
                  <div>
                    <p className="eyebrow">
                      {lineage.post.status === "final_approved" ? "Frozen final" : "Working revision"}
                    </p>
                    <h3>{lineage.post.title}</h3>
                    <p>
                      Revision {currentRevision.sequence} | {humanize(currentRevision.authorshipState)}
                    </p>
                  </div>
                  <span className={`status-pill ${lineage.post.status === "final_approved" ? "" : "attention"}`}>
                    {humanize(lineage.post.status)}
                  </span>
                </div>

                {lineage.supportingMaterial.length > 0 ? (
                  <div className="post-support-list" aria-label="Post supporting material">
                    {lineage.supportingMaterial.map((item) => (
                      <span className="post-support-chip" key={item.relationshipId}>
                        {humanize(item.role)}
                      </span>
                    ))}
                  </div>
                ) : (
                  <p className="post-standing-warning">
                    No supporting material is linked. WorkLore cannot infer standing from the topic alone.
                  </p>
                )}

                <label className="field-label" htmlFor="post-editor">Post text</label>
                <textarea
                  id="post-editor"
                  className="post-editor"
                  value={editorText}
                  onChange={(event) => {
                    setEditorText(event.target.value);
                    setChallenge(null);
                  }}
                  readOnly={lineage.post.status === "final_approved"}
                />

                {lineage.post.status === "working" ? (
                  <div className="support-actions">
                    <button
                      className="secondary-button compact"
                      disabled={busy || !unsavedChanges || !editorText.trim()}
                      onClick={() => void saveRevision()}
                    >
                      Save revision
                    </button>
                    <button
                      className="secondary-button compact"
                      disabled={busy || !editorText.trim()}
                      onClick={() => void challengeDraft()}
                    >
                      Challenge draft
                    </button>
                    <button
                      className="primary-button compact"
                      disabled={busy || !canApprove}
                      onClick={() => void approveFinal()}
                    >
                      Approve final
                    </button>
                  </div>
                ) : (
                  <div className="next-step-card">
                    <h4>Human publication boundary</h4>
                    <p>
                      This exact Revision is frozen as the approved final. Publish it manually outside
                      WorkLore, then record that publication and its outcomes in Insights.
                    </p>
                    <button className="secondary-button compact" onClick={() => void copyApprovedText()}>
                      Copy approved text
                    </button>
                  </div>
                )}

                {unsavedChanges ? (
                  <p className="post-standing-warning">
                    Unsaved edits are not part of canonical revision history and cannot be final-approved.
                  </p>
                ) : null}
              </section>

              {challenge ? (
                <section className="workspace-panel challenge-panel" aria-labelledby="challenge-heading">
                  <div className="panel-heading-row">
                    <div>
                      <p className="eyebrow">Explainable checks</p>
                      <h3 id="challenge-heading">Challenge</h3>
                    </div>
                    <span className={`status-pill ${challenge.confidentiality.state === "ready" ? "" : "attention"}`}>
                      privacy: {humanize(challenge.confidentiality.state)}
                    </span>
                  </div>

                  <div className="challenge-grid">
                    <div className="next-step-card">
                      <h4>Standing and evidence</h4>
                      {standingLinks.length > 0 ? (
                        <p>{standingLinks.length} first-party evidence or Story link(s) support this Post.</p>
                      ) : (
                        <p>
                          No Story, Proof Point, Evidence, or Evidence Source is linked. Treat claims of
                          expertise or firsthand experience as unsupported until you connect standing.
                        </p>
                      )}
                    </div>
                    <div className="next-step-card">
                      <h4>Writing patterns</h4>
                      <p>
                        {challenge.lint.findings.length === 0
                          ? "No deterministic pattern findings."
                          : `${challenge.lint.findings.length} deterministic finding(s) to review.`}
                      </p>
                    </div>
                    <div className="next-step-card">
                      <h4>Confidentiality</h4>
                      <p>
                        {challenge.confidentiality.state !== "ready"
                          ? `${challenge.confidentiality.unresolvedRisks.length} unresolved privacy risk(s) require review.`
                          : publicSafeMatchesEditor
                            ? "The exact saved text is already public-safe."
                            : "A different public-safe version is available and must become a saved Revision before approval."}
                      </p>
                    </div>
                  </div>

                  {challenge.lint.findings.length > 0 ? (
                    <div className="challenge-findings">
                      {challenge.lint.findings.map((finding) => (
                        <article className="challenge-finding" key={`${finding.ruleId}-${finding.startOffset ?? "all"}`}>
                          <strong>{humanize(finding.category)} | {finding.severity}</strong>
                          <p>{finding.reason}</p>
                          {finding.remediation ? <small>{finding.remediation}</small> : null}
                        </article>
                      ))}
                    </div>
                  ) : null}

                  {challenge.lint.unsupportedWritingRules.length > 0 ? (
                    <p className="post-standing-warning">
                      {challenge.lint.unsupportedWritingRules.length} active Writing Rule(s) are advisory only
                      and cannot yet be checked deterministically.
                    </p>
                  ) : null}

                  {challenge.confidentiality.unresolvedRisks.length > 0 ? (
                    <div className="challenge-findings">
                      {challenge.confidentiality.unresolvedRisks.map((risk, index) => (
                        <article className="challenge-finding" key={`${risk.source}-${risk.reviewItemId ?? index}`}>
                          <strong>{humanize(risk.source)}</strong>
                          <p>{risk.reason}</p>
                        </article>
                      ))}
                    </div>
                  ) : null}

                  {challenge.confidentiality.publicSafeText !== challenge.confidentiality.originalText ? (
                    <div className="public-safe-preview">
                      <label className="field-label" htmlFor="public-safe-preview">Derived public-safe preview</label>
                      <textarea
                        id="public-safe-preview"
                        value={challenge.confidentiality.publicSafeText}
                        readOnly
                      />
                      <p>
                        This preview never overwrites private Revision truth. Load it into the editor only
                        if you want to create a new human-reviewed Revision.
                      </p>
                      <button
                        className="secondary-button compact"
                        disabled={challenge.confidentiality.state !== "ready"}
                        onClick={loadPublicSafeText}
                      >
                        Load public-safe text into editor
                      </button>
                    </div>
                  ) : null}

                  {challengeMatchesEditor &&
                  !unsavedChanges &&
                  challenge.confidentiality.state === "ready" &&
                  publicSafeMatchesEditor ? (
                    <p className="challenge-ready">
                      The exact saved Revision has crossed the deterministic challenge and confidentiality gate.
                      Advisory findings remain yours to accept or override.
                    </p>
                  ) : null}
                </section>
              ) : null}
            </>
          )}
        </div>
      </div>

      {notice ? <p className="inline-notice" role="status">{notice}</p> : null}
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </section>
  );
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}
