#!/usr/bin/env python3
"""Wire the Capture-embedded Topic flow into the same generated-Post handoff."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"Expected exactly one match in {path}, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


replace_once(
    "src/components/CapturePanel.tsx",
    'export function CapturePanel({ vaultPath }: { vaultPath: string }) {',
    'export function CapturePanel({\n  vaultPath,\n  onPostGenerated,\n}: {\n  vaultPath: string;\n  onPostGenerated: (postId: string) => void;\n}) {',
)
replace_once(
    "src/components/CapturePanel.tsx",
    '        <TopicPanel vaultPath={vaultPath} topicId={developmentTopicId} onClose={() => setDevelopmentTopicId(null)} />',
    '        <TopicPanel\n          vaultPath={vaultPath}\n          topicId={developmentTopicId}\n          onClose={() => setDevelopmentTopicId(null)}\n          onPostGenerated={onPostGenerated}\n        />',
)
replace_once(
    "src/App.tsx",
    '      case "capture":\n        return <CapturePanel vaultPath={vault!.path} />;',
    '      case "capture":\n        return (\n          <CapturePanel\n            vaultPath={vault!.path}\n            onPostGenerated={() => setActiveView("posts")}\n          />\n        );',
)

print("Wired Capture Topic generation into Posts navigation.")
