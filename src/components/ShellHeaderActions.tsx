import { useState } from "react";
import { QuickStartGuide } from "./QuickStartGuide";

export function ShellHeaderActions({
  onOpenSettings,
  onOpenVault,
  onCloseVault,
}: {
  onOpenSettings: () => void;
  onOpenVault: () => void;
  onCloseVault: () => void;
}) {
  const [helpOpen, setHelpOpen] = useState(false);

  return (
    <>
      <div className="header-actions shell-header-actions" aria-label="Application actions">
        <button
          className="shell-icon-button help"
          type="button"
          aria-label="Open quick-start help"
          title="Quick start and help"
          onClick={() => setHelpOpen(true)}
        >
          <span className="help-mark" aria-hidden="true">?</span>
        </button>
        <button
          className="shell-icon-button settings"
          type="button"
          aria-label="Open AI settings"
          title="AI settings and providers"
          onClick={onOpenSettings}
        >
          <SettingsIcon />
        </button>
        <button
          className="shell-icon-button vault"
          type="button"
          aria-label="Open another vault"
          title="Open another vault"
          onClick={onOpenVault}
        >
          <FolderIcon />
        </button>
        <button
          className="shell-icon-button neutral"
          type="button"
          aria-label="Close current vault"
          title="Close current vault"
          onClick={onCloseVault}
        >
          <CloseIcon />
        </button>
      </div>
      <QuickStartGuide open={helpOpen} onClose={() => setHelpOpen(false)} />
    </>
  );
}

function SettingsIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <circle cx="12" cy="12" r="3" />
      <path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6 7 7M17 17l1.4 1.4M18.4 5.6 17 7M7 17l-1.4 1.4" />
    </svg>
  );
}

function FolderIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <path d="M3.5 7.5h6l2-2h9v12.8a1.7 1.7 0 0 1-1.7 1.7H5.2a1.7 1.7 0 0 1-1.7-1.7V7.5Z" />
      <path d="M3.5 9.5h17" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <path d="M6 6l12 12M18 6L6 18" />
    </svg>
  );
}
