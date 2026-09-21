import { useEffect, useState } from "react";
import type {
  DiscoveryFeedbackVerdict,
  DiscoveryFreshness,
  DiscoveryOpportunity,
} from "../domain/discovery";
import { errorMessage } from "../domain/types";
import {
  developDiscoveryTopic,
  dismissDiscoveryOpportunity,
  listDiscoveryOpportunities,
  openDiscoverySource,
  recordDiscoveryFeedback,
  saveDiscoveryInspiration,
  scanDiscovery,
} from "../lib/workloreApi";
import "../discovery.css";

const FEEDBACK_REASONS = [
  ["strong_personal_angle", "Strong personal angle"],
  ["weak_standing", "Weak standing"],
  ["too_generic", "Too generic"],
  ["audience_mismatch", "Audience mismatch"],
  ["overdone", "I have covered this enough"],
  ["bad_timing", "Bad timing"],
  ["source_quality", "Source quality concern"],
  ["not_interested", "Not interesting to me"],
  ["other", "Other"],
] as const;

export function DiscoveryPanel({
  vaultPath,
  onTopicDeveloped,
}: {
  vaultPath: string;
  onTopicDeveloped: (topicId: string) => void;
}) {
  const [focus, setFocus] = useState("");
  const [freshness, setFreshness] = useState<DiscoveryFreshness>("week");
  const [opportunities, setOpportunities] = useState<DiscoveryOpportunity[]>([]);
  const [busy, setBusy] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [vaultPath]);

  async function refresh() {
    try {
      setOpportunities(await listDiscoveryOpportunities(vaultPath, 20));
    } catch (caught) {
      setError(errorMessage(caught));
    }
  }

  async function scan() {
    setBusy("Scanning current web results");
    setError(null);
    setNotice(null);
    try {
      const result = await scanDiscovery(vaultPath, {
        focus: focus.trim(),
        freshness,
        maxResults: 12,
      });
      await refresh();
      const count = result.feedbackExamplesUsed;
      setNotice(
        count > 0
          ? "Scan complete. " + count + " prior feedback example" + (count === 1 ? "" : "s") + " informed qualification."
          : "Scan complete. No prior feedback examples matched this run yet.",
      );
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function saveInspiration(opportunityId: string) {
    setBusy("Saving Inspiration");
    setError(null);
    try {
      await saveDiscoveryInspiration(vaultPath, opportunityId);
      await refresh();
      setNotice("Saved as external Inspiration. It remains context, not evidence about you.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  async function dismiss(opportunityId: string) {
    setBusy("Dismissing opportunity");
    setError(null);
    try {
      await dismissDiscoveryOpportunity(vaultPath, opportunityId);
      await refresh();
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(null);
    }
  }

  return (
    <section className="discovery-panel" aria-labelledby="discovery-heading">
      <div className="discovery-heading">
        <div>
          <p className="eyebrow">External awareness</p>
          <h3 id="discovery-heading">Find timely topics</h3>
        </div>
        <span className="discovery-boundary">Opportunity finder, not a news feed</span>
      </div>

      <p className="discovery-intro">
        Scan current public material, then qualify it against your Themes, standing, Target Context,
        recent Topics, and prior discovery feedback. Search results remain external context until you
        explicitly develop them.
      </p>

      <div className="discovery-scan-row">
        <label>
          Focus
          <input
            value={focus}
            disabled={busy !== null}
            onChange={(event) => setFocus(event.target.value)}
            placeholder="Optional. Leave blank to use active Themes."
          />
        </label>
        <label>
          Freshness
          <select
            value={freshness}
            disabled={busy !== null}
            onChange={(event) => setFreshness(event.target.value as DiscoveryFreshness)}
          >
            <option value="day">Past 24 hours</option>
            <option value="week">Past 7 days</option>
            <option value="month">Past 31 days</option>
          </select>
        </label>
        <button
          className="primary-button compact"
          type="button"
          disabled={busy !== null}
          onClick={() => void scan()}
        >
          Scan now
        </button>
      </div>

      <p className="discovery-privacy-note">
        Discovery uses the separately configured Brave Search key. WorkLore privacy-preflights the
        query before sending it and does not send your Story or Proof Point text to the search API.
      </p>
      {busy ? <p className="discovery-status">{busy}</p> : null}
      {notice ? <p className="discovery-notice" role="status">{notice}</p> : null}
      {error ? <p className="inline-error" role="alert">{error}</p> : null}

      {opportunities.length === 0 ? (
        <div className="compact-empty-state">
          No discovery opportunities yet. Run a scan when you want a current hook for your existing
          professional material.
        </div>
      ) : (
        <div className="discovery-list">
          {opportunities.map((opportunity) => (
            <DiscoveryOpportunityCard
              key={opportunity.opportunityId}
              vaultPath={vaultPath}
              opportunity={opportunity}
              disabled={busy !== null}
              onRefresh={refresh}
              onBusy={setBusy}
              onError={setError}
              onNotice={setNotice}
              onSaveInspiration={saveInspiration}
              onDismiss={dismiss}
              onTopicDeveloped={onTopicDeveloped}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function DiscoveryOpportunityCard({
  vaultPath,
  opportunity,
  disabled,
  onRefresh,
  onBusy,
  onError,
  onNotice,
  onSaveInspiration,
  onDismiss,
  onTopicDeveloped,
}: {
  vaultPath: string;
  opportunity: DiscoveryOpportunity;
  disabled: boolean;
  onRefresh: () => Promise<void>;
  onBusy: (value: string | null) => void;
  onError: (value: string | null) => void;
  onNotice: (value: string | null) => void;
  onSaveInspiration: (opportunityId: string) => Promise<void>;
  onDismiss: (opportunityId: string) => Promise<void>;
  onTopicDeveloped: (topicId: string) => void;
}) {
  const [developing, setDeveloping] = useState(false);
  const [topicTitle, setTopicTitle] = useState(opportunity.title);
  const [topicSummary, setTopicSummary] = useState("");
  const [verdict, setVerdict] = useState<DiscoveryFeedbackVerdict>("good_candidate");
  const [reasons, setReasons] = useState<string[]>([]);
  const [feedbackNote, setFeedbackNote] = useState("");

  function toggleReason(reason: string) {
    setReasons((current) =>
      current.includes(reason)
        ? current.filter((item) => item !== reason)
        : [...current, reason],
    );
  }

  async function saveFeedback() {
    onBusy("Saving discovery feedback");
    onError(null);
    try {
      await recordDiscoveryFeedback(vaultPath, {
        opportunityId: opportunity.opportunityId,
        verdict,
        reasons,
        note: feedbackNote.trim(),
      });
      setFeedbackNote("");
      setReasons([]);
      onNotice(
        "Feedback saved for future discovery runs. It does not modify Voice or silently create a permanent rule.",
      );
      await onRefresh();
    } catch (caught) {
      onError(errorMessage(caught));
    } finally {
      onBusy(null);
    }
  }

  async function createTopic() {
    if (!topicTitle.trim() || !topicSummary.trim()) {
      onError("Add both a Topic title and your own summary before developing this opportunity.");
      return;
    }
    onBusy("Creating Topic from reviewed discovery opportunity");
    onError(null);
    try {
      const result = await developDiscoveryTopic(vaultPath, {
        opportunityId: opportunity.opportunityId,
        title: topicTitle.trim(),
        summary: topicSummary.trim(),
      });
      onNotice("Topic created with external Inspiration and matched standing kept as separate provenance.");
      onTopicDeveloped(result.topicId);
    } catch (caught) {
      onError(errorMessage(caught));
    } finally {
      onBusy(null);
    }
  }

  async function openSource(url: string) {
    onError(null);
    try {
      await openDiscoverySource(vaultPath, opportunity.opportunityId, url);
    } catch (caught) {
      onError(errorMessage(caught));
    }
  }

  const inactive = opportunity.status === "dismissed";

  return (
    <article className={"discovery-card " + (inactive ? "dismissed" : "")}>
      <div className="discovery-card-heading">
        <div>
          <span className="discovery-status-pill">{humanize(opportunity.status)}</span>
          <h4>{opportunity.title}</h4>
        </div>
        <span className="discovery-source-count">
          {opportunity.sources.length} source{opportunity.sources.length === 1 ? "" : "s"}
        </span>
      </div>

      {opportunity.summary ? <p>{opportunity.summary}</p> : null}

      <div className="discovery-explanation-grid">
        <Explanation title="Why now" body={opportunity.whyNow} />
        <Explanation title="Possible angle" body={opportunity.possibleAngle} />
      </div>

      <MatchRow label="Themes" values={opportunity.themeMatches.map((item) => item.label)} />
      <MatchRow label="Standing" values={opportunity.standingMatches.map((item) => item.label)} />
      <MatchRow label="Audience" values={opportunity.audienceMatches.map((item) => item.label)} />

      {opportunity.feedbackAdjustment ? (
        <p className="discovery-learning-note">
          <strong>Prior feedback:</strong> {opportunity.feedbackAdjustment}
        </p>
      ) : null}

      {opportunity.concerns.length > 0 ? (
        <div className="discovery-concerns">
          <strong>Watch-outs</strong>
          <ul>
            {opportunity.concerns.map((concern) => <li key={concern}>{concern}</li>)}
          </ul>
        </div>
      ) : null}

      <div className="discovery-sources">
        {opportunity.sources.map((source) => (
          <button
            key={source.url}
            className="discovery-source-button"
            type="button"
            disabled={disabled}
            title={source.url}
            onClick={() => void openSource(source.url)}
          >
            <strong>{source.domain || "Source"}</strong>
            <span>{source.title}</span>
            {source.age ? <small>{source.age}</small> : null}
          </button>
        ))}
      </div>

      {!inactive ? (
        <div className="discovery-actions">
          <button
            className="secondary-button compact"
            type="button"
            disabled={disabled || opportunity.inspirationId !== null}
            onClick={() => void onSaveInspiration(opportunity.opportunityId)}
          >
            {opportunity.inspirationId ? "Inspiration saved" : "Save as Inspiration"}
          </button>
          {opportunity.topicId ? (
            <button
              className="primary-button compact"
              type="button"
              disabled={disabled}
              onClick={() => onTopicDeveloped(opportunity.topicId as string)}
            >
              Open Topic
            </button>
          ) : (
            <button
              className="primary-button compact"
              type="button"
              disabled={disabled}
              onClick={() => setDeveloping((value) => !value)}
            >
              Develop Topic
            </button>
          )}
          <button
            className="quiet-button compact"
            type="button"
            disabled={disabled}
            onClick={() => void onDismiss(opportunity.opportunityId)}
          >
            Dismiss
          </button>
        </div>
      ) : null}

      {developing && !opportunity.topicId ? (
        <div className="discovery-develop-editor">
          <strong>Confirm your author intent</strong>
          <p>
            The external headline is only a candidate. Review the title and write the point you
            actually want to make. This explicit confirmation is what turns discovery into Topic
            author intent.
          </p>
          <label>
            Topic title
            <input value={topicTitle} onChange={(event) => setTopicTitle(event.target.value)} />
          </label>
          <label>
            Your summary / point of view
            <textarea
              rows={4}
              value={topicSummary}
              placeholder={opportunity.possibleAngle}
              onChange={(event) => setTopicSummary(event.target.value)}
            />
          </label>
          <div className="discovery-actions">
            <button
              className="primary-button compact"
              type="button"
              disabled={disabled || !topicTitle.trim() || !topicSummary.trim()}
              onClick={() => void createTopic()}
            >
              Create Topic
            </button>
            <button
              className="quiet-button compact"
              type="button"
              disabled={disabled}
              onClick={() => setDeveloping(false)}
            >
              Cancel
            </button>
          </div>
        </div>
      ) : null}

      <details className="discovery-feedback">
        <summary>Tell WorkLore why this is or is not a good candidate</summary>
        <div className="discovery-feedback-body">
          <label>
            Verdict
            <select
              value={verdict}
              disabled={disabled}
              onChange={(event) => setVerdict(event.target.value as DiscoveryFeedbackVerdict)}
            >
              <option value="good_candidate">Good candidate</option>
              <option value="not_now">Not now</option>
              <option value="not_for_me">Not for me</option>
            </select>
          </label>
          <div className="discovery-reasons" aria-label="Feedback reasons">
            {FEEDBACK_REASONS.map(([value, label]) => (
              <label key={value}>
                <input
                  type="checkbox"
                  checked={reasons.includes(value)}
                  disabled={disabled}
                  onChange={() => toggleReason(value)}
                />
                <span>{label}</span>
              </label>
            ))}
          </div>
          <label>
            Why? <span className="optional-label">Optional</span>
            <textarea
              rows={3}
              value={feedbackNote}
              disabled={disabled}
              placeholder="For example: Relevant, but I only want AI news when I have a concrete lesson from building with it."
              onChange={(event) => setFeedbackNote(event.target.value)}
            />
          </label>
          <button
            className="secondary-button compact"
            type="button"
            disabled={disabled}
            onClick={() => void saveFeedback()}
          >
            Save feedback
          </button>
          <p className="discovery-privacy-note">
            Raw feedback is retained locally alongside normalized discovery signals. It trains
            discovery qualification only, never canonical Voice.
          </p>
        </div>
      </details>
    </article>
  );
}

function Explanation({ title, body }: { title: string; body: string }) {
  return (
    <div className="discovery-explanation">
      <strong>{title}</strong>
      <p>{body}</p>
    </div>
  );
}

function MatchRow({ label, values }: { label: string; values: string[] }) {
  return (
    <div className="discovery-match-row">
      <strong>{label}</strong>
      <span>{values.length > 0 ? values.join(" | ") : "No match found"}</span>
    </div>
  );
}

function humanize(value: string): string {
  return value.replaceAll("_", " ");
}
