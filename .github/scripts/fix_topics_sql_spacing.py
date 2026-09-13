from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SERVICE = ROOT / "src-tauri/src/services/topic_service.rs"

text = SERVICE.read_text(encoding="utf-8")
needle = "\\" + "\n"
replacement = " \\" + "\n"
count = text.count(needle)
if count < 1:
    raise SystemExit("Expected escaped-newline SQL joins in topic_service.rs")
text = text.replace(needle, replacement)

old = '''        let captured = load_topic(&path, &topic_id).unwrap();\n        assert_eq!(captured.lifecycle, TopicLifecycle::Captured);\n'''
new = '''        let captured = load_topic(&path, &topic_id).unwrap();\n        let listed = list_topics(&path).unwrap();\n        assert!(listed.iter().any(|topic| topic.topic_id == topic_id));\n        assert_eq!(captured.lifecycle, TopicLifecycle::Captured);\n'''
if text.count(old) != 1:
    raise SystemExit("Expected captured Topic test anchor exactly once")
text = text.replace(old, new, 1)

old = '''        let updated_theme = update_theme(\n            &path,\n            UpdateThemeRequest {\n'''
new = '''        let listed_themes = list_themes(&path).unwrap();\n        assert_eq!(listed_themes.len(), 2);\n\n        let updated_theme = update_theme(\n            &path,\n            UpdateThemeRequest {\n'''
if text.count(old) != 1:
    raise SystemExit("Expected Theme list test anchor exactly once")
text = text.replace(old, new, 1)

SERVICE.write_text(text, encoding="utf-8")
print(f"Normalized {count} escaped-newline SQL joins and strengthened list coverage.")
