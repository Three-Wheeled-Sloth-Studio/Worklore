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
