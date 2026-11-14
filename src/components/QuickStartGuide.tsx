import { useEffect, useRef } from "react";
import { QUICK_START_SECTIONS } from "../help/quickStartContent";
import "../quick-start.css";

export function QuickStartGuide({ open, onClose }: { open: boolean; onClose: () => void }) {
  const dialogRef = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (open && !dialog.open) {
      dialog.showModal();
      return;
    }

    if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className="quick-start-dialog"
      aria-labelledby="quick-start-title"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClose={onClose}
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <section className="quick-start-panel">
        <header className="quick-start-header">
          <div>
            <p className="eyebrow">Help</p>
            <h2 id="quick-start-title">Quick start</h2>
            <p className="quick-start-lede">Build professional memory first. Draft from it later.</p>
          </div>
          <button
            className="shell-icon-button"
            type="button"
            aria-label="Close quick start"
            title="Close quick start"
            onClick={onClose}
          >
            <CloseIcon />
          </button>
        </header>

        <div className="quick-start-sections">
          {QUICK_START_SECTIONS.map((section, index) => (
            <details key={section.id} className="quick-start-section" open={index === 0}>
              <summary>
                <span className="quick-start-step">{index + 1}</span>
                <span className="quick-start-summary-copy">
                  <strong>{section.title}</strong>
                  <small>{section.summary}</small>
                </span>
              </summary>
              <div className="quick-start-detail">
                <ol>
                  {section.steps.map((step) => (
                    <li key={step}>{step}</li>
                  ))}
                </ol>
                {section.tip ? <p className="quick-start-tip">{section.tip}</p> : null}
              </div>
            </details>
          ))}
        </div>
      </section>
    </dialog>
  );
}

function CloseIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
      <path d="M6 6l12 12M18 6L6 18" />
    </svg>
  );
}
