import { useEffect, useState } from "react";
import { errorMessage } from "../domain/types";
import {
  clearPostAudienceIntent,
  getAudienceGoals,
  getPostAudienceIntent,
  savePostAudienceIntent,
  type PostAudienceIntent,
} from "../lib/audienceGoalsApi";

export function PostAudiencePanel({ vaultPath, postId }: { vaultPath: string; postId: string }) {
  const [vaultAudience, setVaultAudience] = useState("");
  const [override, setOverride] = useState<PostAudienceIntent | null>(null);
  const [audience, setAudience] = useState("");
  const [signal, setSignal] = useState("");
  const [editing, setEditing] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setEditing(false);
    setError(null);
    setNotice(null);
    setOverride(null);
    void Promise.all([getAudienceGoals(vaultPath), getPostAudienceIntent(vaultPath, postId)])
      .then(([goals, selected]) => {
        if (cancelled) return;
        setVaultAudience(goals.primaryAudience);
        setOverride(selected);
        setAudience(selected?.primaryAudience ?? goals.primaryAudience);
        setSignal(selected?.desiredSignal ?? "");
      })
      .catch((caught: unknown) => { if (!cancelled) setError(errorMessage(caught)); });
    return () => { cancelled = true; };
  }, [vaultPath, postId]);

  async function save() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const result = await savePostAudienceIntent(vaultPath, {
        postId, primaryAudience: audience.trim(), desiredSignal: signal.trim(), updatedAt: "",
      });
      setOverride(result);
      setEditing(false);
      setNotice("Post audience saved locally.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally { setBusy(false); }
  }

  async function useVaultDefault() {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      await clearPostAudienceIntent(vaultPath, postId);
      setOverride(null);
      setAudience(vaultAudience);
      setSignal("");
      setEditing(false);
      setNotice("This post now uses the vault audience goals.");
    } catch (caught) {
      setError(errorMessage(caught));
    } finally { setBusy(false); }
  }

  return <section className="post-audience-intent" aria-label="Post audience intent">
    <div className="post-audience-heading">
      <strong>Post audience</strong>
      {!editing ? <button type="button" className="quiet-button compact" onClick={() => setEditing(true)}>Set for this post</button> : null}
    </div>
    {!editing ? <p>{override?.primaryAudience || vaultAudience || "Not set"}
      {override?.desiredSignal ? ` | Reader signal: ${override.desiredSignal}` : ""}
      {override ? " (post override)" : " (vault default)"}</p> : <div className="post-audience-edit">
      <label>Primary reader <input maxLength={200} value={audience} onChange={(e) => setAudience(e.target.value)} /></label>
      <label>What should this reader understand? <textarea rows={2} maxLength={1000} value={signal} onChange={(e) => setSignal(e.target.value)} /></label>
      <div className="compact-action-row">
        <button type="button" className="primary-button compact" disabled={busy || !audience.trim()} onClick={() => void save()}>Save</button>
        <button type="button" className="quiet-button compact" disabled={busy} onClick={() => setEditing(false)}>Cancel</button>
        {override ? <button type="button" className="quiet-button compact" disabled={busy} onClick={() => void useVaultDefault()}>Use vault goals</button> : null}
      </div>
    </div>}
    {notice ? <p role="status">{notice}</p> : null}
    {error ? <p role="alert">{error}</p> : null}
  </section>;
}
