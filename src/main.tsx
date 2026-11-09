import { invoke } from "@tauri-apps/api/core";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { WindowControls } from "./components/WindowControls";
import "./guided-interview.css";
import "./legal-notice.css";
import "./performance.css";
import "./privacy-review.css";
import "./story-bank.css";
import "./story-candidates.css";

function handleExternalLinkClick(event: MouseEvent) {
  if (event.defaultPrevented) {
    return;
  }

  const target = event.target;
  if (!(target instanceof Element)) {
    return;
  }

  const anchor = target.closest('a[target="_blank"][href^="https://"]');
  if (!(anchor instanceof HTMLAnchorElement)) {
    return;
  }

  event.preventDefault();
  void invoke<void>("open_external_url", { url: anchor.href }).catch((caught) => {
    console.error("WorkLore could not open the external link.", caught);
  });
}

document.addEventListener("click", handleExternalLinkClick);

const root = document.getElementById("root");

if (!root) {
  throw new Error("WorkLore could not find its application root.");
}

createRoot(root).render(
  <StrictMode>
    <App />
    <WindowControls />
  </StrictMode>,
);
