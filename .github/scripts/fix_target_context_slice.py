from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    text = path.read_text(encoding="utf-8")
    if old not in text:
        raise RuntimeError(f"Expected {label} block was not found in {path}.")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


rust = ROOT / "src-tauri/src/services/target_context_service.rs"
replace_once(
    rust,
    '''pub enum TargetContextLifecycle {
    Active,
    Archived,
}
''',
    '''pub enum TargetContextLifecycle {
    Active,
    Stale,
    Archived,
}
''',
    "Target Context lifecycle enum",
)
replace_once(
    rust,
    '''        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
''',
    '''        match self {
            Self::Active => "active",
            Self::Stale => "stale",
            Self::Archived => "archived",
        }
''',
    "Target Context lifecycle serialization",
)
replace_once(
    rust,
    '''        match value {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
''',
    '''        match value {
            "active" => Ok(Self::Active),
            "stale" => Ok(Self::Stale),
            "archived" => Ok(Self::Archived),
''',
    "Target Context lifecycle parsing",
)
replace_once(
    rust,
    '''                title: "Director of Product — synthetic".into(),
                lifecycle: TargetContextLifecycle::Active,
''',
    '''                title: "Director of Product — synthetic".into(),
                lifecycle: TargetContextLifecycle::Stale,
''',
    "Target Context stale lifecycle test update",
)
replace_once(
    rust,
    '''        assert_eq!(updated.target_id, before.target_id);
        assert_eq!(updated.revision, before.revision + 1);
        let reopened = load_target_context(&path, &before.target_id).expect("reopen");
''',
    '''        assert_eq!(updated.target_id, before.target_id);
        assert_eq!(updated.lifecycle, TargetContextLifecycle::Stale);
        assert_eq!(updated.revision, before.revision + 1);
        let reopened = load_target_context(&path, &before.target_id).expect("reopen");
        assert_eq!(reopened.lifecycle, TargetContextLifecycle::Stale);
''',
    "Target Context stale lifecycle assertions",
)

typescript = ROOT / "src/domain/types.ts"
replace_once(
    typescript,
    'export type TargetContextLifecycle = "active" | "archived";\n',
    'export type TargetContextLifecycle = "active" | "stale" | "archived";\n',
    "TypeScript Target Context lifecycle",
)

panel = ROOT / "src/components/TargetContextPanel.tsx"
replace_once(
    panel,
    '''const LIFECYCLES: Array<{ value: TargetContextLifecycle; label: string }> = [
  { value: "active", label: "Active" },
  { value: "archived", label: "Archived" },
];
''',
    '''const LIFECYCLES: Array<{ value: TargetContextLifecycle; label: string }> = [
  { value: "active", label: "Active" },
  { value: "stale", label: "Stale" },
  { value: "archived", label: "Archived" },
];
''',
    "Target Context lifecycle UI options",
)

print("Added stale Target Context lifecycle across Rust, TypeScript, UI, and lifecycle proof coverage.")
