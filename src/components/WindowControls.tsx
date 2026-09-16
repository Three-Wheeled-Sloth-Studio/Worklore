import { getCurrentWindow } from "@tauri-apps/api/window";
import { HelpLauncher } from "./HelpLauncher";

export function WindowControls() {
  async function closeApplication() {
    await getCurrentWindow().close();
  }

  return (
    <div className="window-controls" aria-label="Application controls">
      <HelpLauncher />
      <button
        className="shell-icon-button window-close-button"
        type="button"
        aria-label="Exit WorkLore"
        title="Exit WorkLore"
        onClick={() => void closeApplication()}
      >
        <CloseIcon />
      </button>
    </div>
  );
}

function CloseIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <path d="M6 6l12 12M18 6L6 18" />
    </svg>
  );
}
