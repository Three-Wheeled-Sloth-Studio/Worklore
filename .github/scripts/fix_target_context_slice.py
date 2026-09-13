from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
path = ROOT / "src-tauri/src/services/target_context_service.rs"
text = path.read_text(encoding="utf-8")
old = '''        statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?
    };
'''
new = '''        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        rows
    };
'''
if old not in text:
    raise RuntimeError("Expected target-context list query block was not generated.")
text = text.replace(old, new, 1)
path.write_text(text, encoding="utf-8", newline="\n")
print("Normalized generated Target Context query lifetime.")
