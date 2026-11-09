import { useEffect, useState } from "react";
import type { AudienceGoals } from "../lib/audienceGoalsApi";
import { getAudienceGoals, saveAudienceGoals } from "../lib/audienceGoalsApi";
import { errorMessage } from "../domain/types";

const BLANK: AudienceGoals = {
  primaryAudience: "",
  secondaryAudiences: [],
  targetRoles: [],
  targetIndustries: [],
  positioning: "",
  jobSearchDisclosure: "do_not_mention",
  updatedAt: "",
};

function parseLines(value: string): string[] {
  return value.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
}

export function AudienceGoalsPanel({ vaultPath }: { vaultPath: string }) {
  const [goals, setGoals] = useState<AudienceGoals>(BLANK);
  const [editing, setEditing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    setEditing(false);
    setError(null);
    setNotice(null);
    let cancelled = false;
    void getAudienceGoals(vaultPath).then((result) => {
      if (!cancelled) setGoals(result);
    }).catch((caught: unknown) => {
      if (!cancelled) setError(errorMessage(caught));
    });
    return () => { cancelled = true; };
  }, [vaultPath]);

  async function save() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      setGoals(await saveAudienceGoals(vaultPath, goals));
      setEditing(false);
      setNotice("Audience goals saved in this vault.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally {
      setBusy(false);
    }
  }

  async function cancel() {
    setEditing(false);
    setError(null);
    try { setGoals(await getAudienceGoals(vaultPath)); }
    catch (caught) { setError(errorMessage(caught)); }
  }

  return (
    <section className="workspace-panel audience-goals-panel" aria-labelledby="audience-goals-title">
      <div className="post-workbench-toolbar">
        <h2 id="audience-goals-title">Audience goals</h2>
        {!editing ? (
          <button className="secondary-button compact" type="button" onClick={() => setEditing(true)}>Edit goals</button>
        ) : (
          <div className="compact-action-row">
            <button className="quiet-button compact" type="button" disabled={busy} onClick={() => void cancel()}>Cancel</button>
            <button className="primary-button compact" type="button" disabled={busy} onClick={() => void save()}>Save goals</button>
          </div>
        )}
      </div>
      {!editing ? (
        <p className="audience-goals-summary">
          {goals.primaryAudience || goals.positioning
            ? [goals.primaryAudience, goals.positioning].filter(Boolean).join(" | ")
            : "No audience goals configured. Optional goals help plan who a post should speak to, without claiming expertise you haven't documented."}
          {goals.targetRoles.length ? ` | Roles: ${goals.targetRoles.join(", ")}` : ""}
          {goals.jobSearchDisclosure === "may_mention" ? " | Job search mention allowed" : " | Do not mention job search"}
        </p>
      ) : (
        <div className="audience-goals-form">
          <label>Primary audience
            <input maxLength={200} value={goals.primaryAudience} onChange={(event) => setGoals({ ...goals, primaryAudience: event.target.value })} placeholder="e.g. Product executives" />
          </label>
          <label>Secondary audiences (one per line)
            <textarea rows={2} value={goals.secondaryAudiences.join("\n")} onChange={(event) => setGoals({ ...goals, secondaryAudiences: parseLines(event.target.value) })} />
          </label>
          <label>Target roles (one per line)
            <textarea rows={2} value={goals.targetRoles.join("\n")} onChange={(event) => setGoals({ ...goals, targetRoles: parseLines(event.target.value) })} />
          </label>
          <label>Target industries (one per line)
            <textarea rows={2} value={goals.targetIndustries.join("\n")} onChange={(event) => setGoals({ ...goals, targetIndustries: parseLines(event.target.value) })} />
          </label>
          <label>Professional positioning
            <textarea rows={2} maxLength={2000} value={goals.positioning} onChange={(event) => setGoals({ ...goals, positioning: event.target.value })} placeholder="What should a reader learn about how you work?" />
          </label>
          <label>Job search disclosure
            <select value={goals.jobSearchDisclosure} onChange={(event) => setGoals({ ...goals, jobSearchDisclosure: event.target.value as AudienceGoals["jobSearchDisclosure"] })}>
              <option value="do_not_mention">Do not mention my job search</option>
              <option value="may_mention">May mention my job search</option>
            </select>
          </label>
          <p className="audience-goals-summary">These are editorial objectives, not evidence about your experience. No cloud request is made when saving.</p>
        </div>
      )}
      {notice ? <p role="status">{notice}</p> : null}
      {error ? <p role="alert">{error}</p> : null}
    </section>
  );
}
