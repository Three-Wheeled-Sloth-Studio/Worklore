from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content, encoding="utf-8")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"Expected exactly one match in {path}, found {count}: {old[:120]!r}")
    write(path, text.replace(old, new, 1))


# rustc E0597: ensure rusqlite MappedRows temporaries drop before their statements.
replace_once(
    "src-tauri/src/services/topic_service.rs",
    '''    let ids = {\n        let mut statement = connection.prepare(\n            "SELECT topic_id FROM topic_candidates ORDER BY updated_at DESC, created_at DESC",\n        )?;\n        statement\n            .query_map([], |row| row.get::<_, String>(0))?\n            .collect::<Result<Vec<_>, _>>()?\n    };\n''',
    '''    let ids = {\n        let mut statement = connection.prepare(\n            "SELECT topic_id FROM topic_candidates ORDER BY updated_at DESC, created_at DESC",\n        )?;\n        let rows = statement\n            .query_map([], |row| row.get::<_, String>(0))?\n            .collect::<Result<Vec<_>, _>>()?;\n        rows\n    };\n''',
)

replace_once(
    "src-tauri/src/services/topic_service.rs",
    '''    let raw = {\n        let mut statement = connection.prepare(\n            "SELECT theme_id,name,description,status,created_at,updated_at,revision\\\n             FROM themes ORDER BY updated_at DESC, created_at DESC",\n        )?;\n        statement\n            .query_map([], |row| {\n                Ok((\n                    row.get::<_, String>(0)?,\n                    row.get::<_, String>(1)?,\n                    row.get::<_, String>(2)?,\n                    row.get::<_, String>(3)?,\n                    row.get::<_, String>(4)?,\n                    row.get::<_, String>(5)?,\n                    row.get::<_, u32>(6)?,\n                ))\n            })?\n            .collect::<Result<Vec<_>, _>>()?\n    };\n''',
    '''    let raw = {\n        let mut statement = connection.prepare(\n            "SELECT theme_id,name,description,status,created_at,updated_at,revision\\\n             FROM themes ORDER BY updated_at DESC, created_at DESC",\n        )?;\n        let rows = statement\n            .query_map([], |row| {\n                Ok((\n                    row.get::<_, String>(0)?,\n                    row.get::<_, String>(1)?,\n                    row.get::<_, String>(2)?,\n                    row.get::<_, String>(3)?,\n                    row.get::<_, String>(4)?,\n                    row.get::<_, String>(5)?,\n                    row.get::<_, u32>(6)?,\n                ))\n            })?\n            .collect::<Result<Vec<_>, _>>()?;\n        rows\n    };\n''',
)

replace_once(
    "src-tauri/src/services/topic_service.rs",
    '''    let raw = {\n        let mut statement = connection.prepare(\n            "SELECT relationship_id,relationship_type,to_id FROM record_relationships\\\n             WHERE from_type='topic_candidate' AND from_id=?1 AND relationship_type LIKE 'topic_%'\\\n             ORDER BY created_at,relationship_id",\n        )?;\n        statement\n            .query_map([topic_id], |row| {\n                Ok((\n                    row.get::<_, String>(0)?,\n                    row.get::<_, String>(1)?,\n                    row.get::<_, String>(2)?,\n                ))\n            })?\n            .collect::<Result<Vec<_>, _>>()?\n    };\n''',
    '''    let raw = {\n        let mut statement = connection.prepare(\n            "SELECT relationship_id,relationship_type,to_id FROM record_relationships\\\n             WHERE from_type='topic_candidate' AND from_id=?1 AND relationship_type LIKE 'topic_%'\\\n             ORDER BY created_at,relationship_id",\n        )?;\n        let rows = statement\n            .query_map([topic_id], |row| {\n                Ok((\n                    row.get::<_, String>(0)?,\n                    row.get::<_, String>(1)?,\n                    row.get::<_, String>(2)?,\n                ))\n            })?\n            .collect::<Result<Vec<_>, _>>()?;\n        rows\n    };\n''',
)

# Keep the Tauri command surface free of artificial dead-code shims.
replace_once(
    "src-tauri/src/commands/topics.rs",
    '''        self, CreateThemeRequest, CreateTopicRequest, ThemeRecordView, TopicLifecycle,\n        TopicLinkTargetView, TopicRecordView, TopicRelationKind, TopicRelationshipMutationResult,\n        TopicTimingClass, UpdateThemeRequest, UpdateTopicRequest,\n''',
    '''        self, CreateThemeRequest, CreateTopicRequest, ThemeRecordView, TopicLinkTargetView,\n        TopicRecordView, TopicRelationKind, TopicRelationshipMutationResult, UpdateThemeRequest,\n        UpdateTopicRequest,\n''',
)
replace_once(
    "src-tauri/src/commands/topics.rs",
    '''\n// Keep these domain enums visible at the command boundary for generated/debug tooling.\n#[allow(dead_code)]\nfn _topic_command_contract(\n    lifecycle: TopicLifecycle,\n    timing: TopicTimingClass,\n) -> (TopicLifecycle, TopicTimingClass) {\n    (lifecycle, timing)\n}\n''',
    "\n",
)

# Keep the TypeScript application API symmetrical with the Rust Theme load command.
replace_once(
    "src/lib/workloreApi.ts",
    '''export async function createTheme(\n  vaultPath: string,\n  request: CreateThemeRequest,\n): Promise<ThemeRecord> {\n  return invoke<ThemeRecord>("create_theme", { vaultPath, request });\n}\n\nexport async function listThemes(vaultPath: string): Promise<ThemeRecord[]> {\n''',
    '''export async function createTheme(\n  vaultPath: string,\n  request: CreateThemeRequest,\n): Promise<ThemeRecord> {\n  return invoke<ThemeRecord>("create_theme", { vaultPath, request });\n}\n\nexport async function getTheme(vaultPath: string, themeId: string): Promise<ThemeRecord> {\n  return invoke<ThemeRecord>("get_theme", { vaultPath, themeId });\n}\n\nexport async function listThemes(vaultPath: string): Promise<ThemeRecord[]> {\n''',
)

print("Applied Topics follow-up validation fixes.")
