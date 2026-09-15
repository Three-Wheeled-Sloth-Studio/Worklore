import { useState } from "react";
import { QuickStartGuide } from "./QuickStartGuide";

export function HelpLauncher() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <button
        className="shell-icon-button help-launcher"
        type="button"
        aria-label="Open quick-start help"
        title="Quick start and help"
        onClick={() => setOpen(true)}
      >
        <span aria-hidden="true">?</span>
      </button>
      <QuickStartGuide open={open} onClose={() => setOpen(false)} />
    </>
  );
}
