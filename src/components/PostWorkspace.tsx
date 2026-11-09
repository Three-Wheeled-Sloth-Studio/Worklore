import { useEffect, useMemo, useState } from "react";
import { InfoButton } from "./InfoButton";
import type { ConfidentialityTransformResult } from "../domain/confidentiality";
import type { PostLineageView, PostRecordView } from "../domain/posts";
import type { LintDraftResult, TopicRecord } from "../domain/types";
import { errorMessage } from "../domain/types";
import { transformConfidentialityForPublicUse } from "../lib/confidentialityApi";
import {
  appendPostRevision,
  approvePostRevision,
  generatePostFromTopic,
  getPostLineage,
  listPosts,
} from "../lib/postApi";
import { getProviderSettings, lintDraft, listTopics } from "../lib/workloreApi";
import "../posts.css";

interface ChallengeResult {
  text: string;
  lint: LintDraftResult;
  confidentiality: ConfidentialityTransformResult;
}

export function PostWorkspace({ vaultPath }: { vaultPath: string }) {
  const [posts, setPosts] = useState<PostRecordView[]>([]);
  const [topics, setTopics] = useState<TopicRecord[]>([]);
  const [selectedTopicId, setSelectedTopicId] = useState("");
  const [selectedPostId, setSelectedPostId] = useState("");
  const [lineage, setLineage] = useState<PostLineageView | null>(null);
  const [editorText, setEditorText] = useState("");
  const [challenge, setChallenge] = useState<ChallengeResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const currentRevision = useMemo(() => {
    if (!lineage) return null;
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
    setSelectedTopicId("");
    setSelectedPostId("");
    setLineage(null);
    setEditorText("");
    setChallenge(null);
    setNotice(null);
    setError(null);
    void loadWorkspace();
  }, [vaultPath]);

  async function loadWorkspace(preferredPostId?: string) {
    setBusy(true);
    setError(null);
    try {
      const [topicResult, postResult] = await Promise.all([
        listTopics(vaultPath),
        listPosts(vaultPath),
      ]);
      setTopics(topicResult);
      setPosts(postResult);
      setSelectedTopicId((current) => current || topicResult[0]?.topicId || "");
      const nextPostId =
        preferredPostId && postResult.some((post) => post.postId === preferredPostId)
          ? preferredPostId
          : postResult[0]?.postId ?? "";
      if (nextPostId) {
        await openPost(nextPostId);
      }
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function refreshPosts(preferredPostId: string) {
    const postResult = await listPosts(vaultPath);
    setPosts(postResult);
    setSelectedPostId(preferredPostId);
  }

  async function openPost(postId: string) {
    if (!postId) {
      setSelectedPostId("");
      setLineage(null);
      setEditorText("");
      setChallenge(null);
      return;
    }
    setError(null);
    try {
      const loaded = await getPostLineage(vaultPath, postId);
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

  async function generateFromTopic() {
    if (!selectedTopicId) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const settings = await getProviderSettings();
      if (!settings.selectedProviderId || !settings.ollamaModelId) {
        throw new Error(
          "Choose an AI provider and model in Settings before generating a Post.",
        );
      }
      const generated = await generatePostFromTopic(vaultPath, {
        topicId: selectedTopicId,
        providerId: settings.selectedProviderId,
        modelId: settings.ollamaModelId,
      });
      const current = generated.lineage.revisions.find(
        (revision) =>
          revision.revisionId === generated.lineage.post.currentRevisionId,
      );
      setLineage(generated.lineage);
      setSelectedPostId(generated.lineage.post.postId);
      setEditorText(current?.text ?? "");
      setChallenge(null);
      await refreshPosts(generated.lineage.post.postId);
      setNotice("Generated from the selected Topic. Review and edit before approval.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function saveRevision() {
    if (!lineage || !currentRevision || !unsavedChanges) return;
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
      const current = updated.revisions.find(
        (revision) => revision.revisionId === updated.post.currentRevisionId,
      );
      setLineage(updated);
      setEditorText(current?.text ?? editorText);
      setChallenge(null);
      await refreshPosts(updated.post.postId);
      setNotice("Revision saved.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function challengeDraft() {
    if (!editorText.trim()) return;
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
        setNotice("Privacy review is still required before approval.");
      } else if (confidentiality.publicSafeText !== editorText) {
        setNotice("A different public-safe version is available below.");
      } else {
        setNotice("Challenge complete.");
      }
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function approveFinal() {
    if (!lineage || !currentRevision || !canApprove) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const approved = await approvePostRevision(vaultPath, {
        postId: lineage.post.postId,
        revisionId: currentRevision.revisionId,
      });
      setLineage(approved);
      await refreshPosts(approved.post.postId);
      setNotice("Final revision approved.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  function loadPublicSafeText() {
    if (!challenge || challenge.confidentiality.state !== "ready") return;
    setEditorText(challenge.confidentiality.publicSafeText);
    setChallenge(null);
    setNotice("Public-safe text loaded. Save it as a new revision, then challenge again.");
  }

  async function copyApprovedText() {
    if (!currentRevision || lineage?.post.status !== "final_approved") return;
    try {
      await navigator.clipboard.writeText(currentRevision.text);
      setNotice("Approved text copied.");
    } catch {
      setNotice("Clipboard access was unavailable. Copy the text manually.");
    }
  }

  return (
    <section className="posts-workspace" aria-labelledby="posts-heading">
      <section className="workspace-panel post-workbench">
        <div className="post-workbench-toolbar">
          <div className="post-toolbar-group post-topic-control">
            <label htmlFor="post-topic-source">Topic</label>
            <select
              id="post-topic-source"
              value={selectedTopicId}
              onChange={(event) => setSelectedTopicId(event.target.value)}
              disabled={busy}
            >
              <option value="">Choose a Topic</option>
              {topics.map((topic) => (
                <option key={topic.topicId} value={topic.topicId}>{topic.title}</option>
              ))}
            </select>
            <span className="post-icon-tooltip" data-tooltip="Generate a draft from the selected Topic">
              <button
                className="post-icon-button primary"
                type="button"
                disabled={busy || !selectedTopicId}
                aria-label="Generate a draft from the selected Topic"
                onClick={() => void generateFromTopic()}
              >
                <SparkleIcon />
              </button>
            </span>
            <InfoButton label="Post generation guidance">
              Pick a Topic and use Generate. The Topic title and summary define the intended point of
              view and may include explicit first-person framing. Linked Stories and Proof Points can
              support additional work-history claims. Generated text is always a model-origin draft
              that requires your review.
            </InfoButton>
          </div>

          <div className="post-toolbar-group post-picker-control">
            <label htmlFor="saved-post">Post</label>
            <select
              id="saved-post"
              value={selectedPostId}
              onChange={(event) => void openPost(event.target.value)}
              disabled={busy || posts.length === 0}
            >
              <option value="">{posts.length === 0 ? "No saved Posts" : "Choose a Post"}</option>
              {posts.map((post) => (
                <option key={post.postId} value={post.postId}>
                  {post.title} · {post.status === "final_approved" ? "final" : `r${post.revision}`}
                </option>
              ))}
            </select>
          </div>
        </div>

        {!lineage || !currentRevision ? (
          <div className="post-empty-state">
            <strong id="posts-heading">Choose a Topic, then Generate.</strong>
          </div>
        ) : (
          <div className="post-editor-surface">
            <div className="post-editor-heading">
              <div>
                <h2 id="posts-heading">{lineage.post.title}</h2>
                <span>
                  r{currentRevision.sequence} · {humanize(currentRevision.authorshipState)}
                  {standingLinks.length > 0 ? ` · ${standingLinks.length} standing link${standingLinks.length === 1 ? "" : "s"}` : ""}
                </span>
              </div>
              <span className={`post-state ${lineage.post.status === "final_approved" ? "final" : ""}`}>
                {lineage.post.status === "final_approved" ? "Final" : "Working"}
              </span>
            </div>

            {lineage.supportingMaterial.length > 0 ? (
              <div className="post-support-list" aria-label="Post support">
                {lineage.supportingMaterial.map((item) => (
                  <span className="post-support-chip" key={item.relationshipId}>
                    {humanize(item.role)}
                  </span>
                ))}
              </div>
            ) : null}

            {standingLinks.length === 0 ? (
              <div className="post-compact-warning" title="No Story or Proof Point is linked to support personal-experience claims.">
                No standing linked. Generation may use explicit Topic assertions but will not invent additional personal-experience claims.
              </div>
            ) : null}

            <textarea
              id="post-editor"
              className="post-editor"
              aria-label="Post text"
              value={editorText}
              onChange={(event) => {
                setEditorText(event.target.value);
                setChallenge(null);
              }}
              readOnly={lineage.post.status === "final_approved"}
            />

            <div className="post-action-bar">
              {lineage.post.status === "working" ? (
                <>
                  <span className="post-icon-tooltip" data-tooltip="Save this edit as a new revision">
                    <button
                      className="post-icon-button"
                      type="button"
                      disabled={busy || !unsavedChanges || !editorText.trim()}
                      aria-label="Save this edit as a new revision"
                      onClick={() => void saveRevision()}
                    >
                      <SaveIcon />
                    </button>
                  </span>
                  <span
                    className="post-icon-tooltip"
                    data-tooltip={unsavedChanges ? "Save edits before checking the draft" : "Check writing rules and privacy"}
                  >
                    <button
                      className="post-icon-button"
                      type="button"
                      disabled={busy || !editorText.trim() || unsavedChanges}
                      aria-label="Check writing rules and privacy"
                      onClick={() => void challengeDraft()}
                    >
                      <ShieldIcon />
                    </button>
                  </span>
                  <span
                    className="post-icon-tooltip"
                    data-tooltip={canApprove ? "Approve this as the final revision" : "Save and check the exact public-safe revision before approval"}
                  >
                    <button
                      className="post-icon-button primary"
                      type="button"
                      disabled={busy || !canApprove}
                      aria-label="Approve this as the final revision"
                      onClick={() => void approveFinal()}
                    >
                      <CheckIcon />
                    </button>
                  </span>
                </>
              ) : (
                <span className="post-icon-tooltip" data-tooltip="Copy the approved Post">
                  <button
                    className="post-icon-button primary"
                    type="button"
                    aria-label="Copy the approved Post"
                    onClick={() => void copyApprovedText()}
                  >
                    <CopyIcon />
                  </button>
                </span>
              )}
              {unsavedChanges ? <span className="post-action-status">Unsaved</span> : null}
            </div>

            {challenge ? (
              <div className="post-challenge-summary" aria-label="Challenge results">
                <span title="First-party standing links">Standing {standingLinks.length}</span>
                <span title="Deterministic writing-pattern findings">Writing {challenge.lint.findings.length}</span>
                <span className={challenge.confidentiality.state === "ready" ? "ready" : "attention"}>
                  Privacy {humanize(challenge.confidentiality.state)}
                </span>

                {challenge.confidentiality.publicSafeText !== challenge.confidentiality.originalText ? (
                  <button
                    className="secondary-button compact"
                    type="button"
                    disabled={challenge.confidentiality.state !== "ready"}
                    onClick={loadPublicSafeText}
                  >
                    Load safe text
                  </button>
                ) : null}

                {challenge.lint.findings.length > 0 || challenge.confidentiality.unresolvedRisks.length > 0 ? (
                  <details className="post-challenge-details">
                    <summary>Review findings</summary>
                    {challenge.lint.findings.map((finding) => (
                      <p key={`${finding.ruleId}-${finding.startOffset ?? "all"}`}>
                        <strong>{humanize(finding.category)}:</strong> {finding.reason}
                      </p>
                    ))}
                    {challenge.confidentiality.unresolvedRisks.map((risk, index) => (
                      <p key={`${risk.source}-${risk.reviewItemId ?? index}`}>
                        <strong>{humanize(risk.source)}:</strong> {risk.reason}
                      </p>
                    ))}
                  </details>
                ) : null}
              </div>
            ) : null}
          </div>
        )}
      </section>

      {notice ? <p className="inline-notice" role="status">{notice}</p> : null}
      {error ? <p className="inline-error" role="alert">{error}</p> : null}
    </section>
  );
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}

function SparkleIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3l1.4 4.6L18 9l-4.6 1.4L12 15l-1.4-4.6L6 9l4.6-1.4zM18 15l.8 2.2L21 18l-2.2.8L18 21l-.8-2.2L15 18l2.2-.8z" /></svg>;
}

function SaveIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 4h11l3 3v13H5zM8 4v6h8V4M8 17h8" /></svg>;
}

function ShieldIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 3l7 3v5c0 4.5-2.8 8-7 10-4.2-2-7-5.5-7-10V6zM9 12l2 2 4-5" /></svg>;
}

function CheckIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12l4 4L19 6" /></svg>;
}

function CopyIcon() {
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M8 8h11v11H8zM5 16H4V5h11v1" /></svg>;
}
