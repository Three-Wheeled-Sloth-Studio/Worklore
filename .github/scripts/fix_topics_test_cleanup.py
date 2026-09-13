from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
path = ROOT / "src-tauri/src/services/topic_service.rs"
text = path.read_text(encoding="utf-8")

old = '''        let theme_targets = list_topic_link_targets(&path, TopicRelationKind::Theme).unwrap();
        assert_eq!(theme_targets.len(), 2);
        fs::remove_dir_all(path).unwrap();
'''
new = '''        let theme_targets = list_topic_link_targets(&path, TopicRelationKind::Theme).unwrap();
        assert_eq!(theme_targets.len(), 2);
        drop(connection);
        fs::remove_dir_all(path).unwrap();
'''

count = text.count(old)
if count != 1:
    raise SystemExit(f"Expected exactly one Topic cleanup match, found {count}.")

path.write_text(text.replace(old, new, 1), encoding="utf-8")
print("Released Topic test SQLite connection before Windows temp-vault cleanup.")
