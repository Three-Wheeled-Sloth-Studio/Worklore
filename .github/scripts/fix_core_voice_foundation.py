from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


path = "src-tauri/src/services/voice_profile_service.rs"
text = read(path)
text = replace_once(
    text,
    '''    fn as_str(self) -> &'static str {\n        match self {\n            Self::Proposed => "proposed",\n            Self::Active => "active",\n            Self::Superseded => "superseded",\n        }\n    }\n\n''',
    "",
    "remove unused CoreVoiceStatus::as_str",
)
for label, block in [
    (
        "ToneModeStatus::parse",
        '''    fn parse(value: &str) -> ServiceResult<Self> {\n        match value {\n            "active" => Ok(Self::Active),\n            "disabled" => Ok(Self::Disabled),\n            "retired" => Ok(Self::Retired),\n            _ => Err(WorkLoreError::InvalidVault(format!(\n                "Unknown Tone Mode status {value}."\n            ))),\n        }\n    }\n''',
    ),
    (
        "VoiceDirectionStatus::parse",
        '''    fn parse(value: &str) -> ServiceResult<Self> {\n        match value {\n            "proposed" => Ok(Self::Proposed),\n            "accepted" => Ok(Self::Accepted),\n            "completed" => Ok(Self::Completed),\n            "retired" => Ok(Self::Retired),\n            _ => Err(WorkLoreError::InvalidVault(format!(\n                "Unknown Voice Direction status {value}."\n            ))),\n        }\n    }\n''',
    ),
    (
        "WritingRuleStatus::parse",
        '''    fn parse(value: &str) -> ServiceResult<Self> {\n        match value {\n            "proposed" => Ok(Self::Proposed),\n            "active" => Ok(Self::Active),\n            "disabled" => Ok(Self::Disabled),\n            "retired" => Ok(Self::Retired),\n            _ => Err(WorkLoreError::InvalidVault(format!(\n                "Unknown Writing Rule status {value}."\n            ))),\n        }\n    }\n''',
    ),
]:
    text = replace_once(text, block, "", f"remove unused {label}")

# rusqlite MappedRows borrows its prepared statement. Rust 1.98 correctly rejects
# returning a final block expression whose temporary may outlive that statement.
# Bind each collection result to a local so the iterator is dropped before scope exit.
text = replace_once(
    text,
    '''    let active_ids = {\n        let mut statement = tx.prepare("SELECT voice_id FROM core_voices WHERE status='active'")?;\n        statement\n            .query_map([], |row| row.get::<_, String>(0))?\n            .collect::<Result<Vec<_>, _>>()?\n    };''',
    '''    let active_ids = {\n        let mut statement = tx.prepare("SELECT voice_id FROM core_voices WHERE status='active'")?;\n        let rows = statement\n            .query_map([], |row| row.get::<_, String>(0))?\n            .collect::<Result<Vec<_>, _>>()?;\n        rows\n    };''',
    "active Core Voice query lifetime",
)

for label, query, mapper in [
    (
        "tone mode list lifetime",
        "SELECT tone_id,name,description,instructions,status,created_at,updated_at,revision\\n         FROM tone_modes ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'disabled' THEN 1 ELSE 2 END,updated_at DESC",
        "map_tone_mode",
    ),
    (
        "voice direction list lifetime",
        "SELECT voice_direction_id,statement,rationale,proposed_by,status,accepted_at,completed_at,created_at,updated_at,revision\\n         FROM voice_directions ORDER BY updated_at DESC,created_at DESC",
        "map_voice_direction",
    ),
    (
        "writing rule list lifetime",
        "SELECT rule_id,name,instruction,status,created_at,updated_at,revision\\n         FROM writing_rules ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'proposed' THEN 1 WHEN 'disabled' THEN 2 ELSE 3 END,updated_at DESC",
        "map_writing_rule",
    ),
]:
    query = query.replace("\\n", "\n")
    old = f'''    let mut statement = connection.prepare(\n        "{query}",\n    )?;\n    statement\n        .query_map([], {mapper})?\n        .collect::<Result<Vec<_>, _>>()\n        .map_err(Into::into)'''
    new = f'''    let mut statement = connection.prepare(\n        "{query}",\n    )?;\n    let rows = statement\n        .query_map([], {mapper})?\n        .collect::<Result<Vec<_>, _>>()?;\n    Ok(rows)'''
    text = replace_once(text, old, new, label)

text = replace_once(
    text,
    '''    let trait_ids = {\n        let mut statement = tx.prepare(\n            "SELECT trait_id FROM core_voice_traits WHERE voice_id=?1 ORDER BY trait_id",\n        )?;\n        statement\n            .query_map([voice_id], |row| row.get::<_, String>(0))?\n            .collect::<Result<Vec<_>, _>>()?\n    };''',
    '''    let trait_ids = {\n        let mut statement = tx.prepare(\n            "SELECT trait_id FROM core_voice_traits WHERE voice_id=?1 ORDER BY trait_id",\n        )?;\n        let rows = statement\n            .query_map([voice_id], |row| row.get::<_, String>(0))?\n            .collect::<Result<Vec<_>, _>>()?;\n        rows\n    };''',
    "activation trait query lifetime",
)
text = replace_once(
    text,
    '''        let statuses = {\n            let mut statement = tx.prepare(\n                "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve\n                 JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id\n                 WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",\n            )?;\n            statement\n                .query_map([&trait_id], |row| {\n                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))\n                })?\n                .collect::<Result<Vec<_>, _>>()?\n        };''',
    '''        let statuses = {\n            let mut statement = tx.prepare(\n                "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve\n                 JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id\n                 WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",\n            )?;\n            let rows = statement\n                .query_map([&trait_id], |row| {\n                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))\n                })?\n                .collect::<Result<Vec<_>, _>>()?;\n            rows\n        };''',
    "activation evidence query lifetime",
)
text = replace_once(
    text,
    '''    let rows = {\n        let mut statement = connection.prepare(\n            "SELECT trait_id,name,value,user_guidance,created_at,updated_at,revision\n             FROM core_voice_traits WHERE voice_id=?1 ORDER BY name,trait_id",\n        )?;\n        statement\n            .query_map([voice_id], |row| {\n                Ok((\n                    row.get::<_, String>(0)?,\n                    row.get::<_, String>(1)?,\n                    row.get::<_, String>(2)?,\n                    row.get::<_, Option<String>>(3)?,\n                    row.get::<_, String>(4)?,\n                    row.get::<_, String>(5)?,\n                    row.get::<_, u32>(6)?,\n                ))\n            })?\n            .collect::<Result<Vec<_>, _>>()?\n    };''',
    '''    let rows = {\n        let mut statement = connection.prepare(\n            "SELECT trait_id,name,value,user_guidance,created_at,updated_at,revision\n             FROM core_voice_traits WHERE voice_id=?1 ORDER BY name,trait_id",\n        )?;\n        let rows = statement\n            .query_map([voice_id], |row| {\n                Ok((\n                    row.get::<_, String>(0)?,\n                    row.get::<_, String>(1)?,\n                    row.get::<_, String>(2)?,\n                    row.get::<_, Option<String>>(3)?,\n                    row.get::<_, String>(4)?,\n                    row.get::<_, String>(5)?,\n                    row.get::<_, u32>(6)?,\n                ))\n            })?\n            .collect::<Result<Vec<_>, _>>()?;\n        rows\n    };''',
    "Core Voice trait rows lifetime",
)
text = replace_once(
    text,
    '''            let evidence = {\n                let mut statement = connection.prepare(\n                    "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve\n                     JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id\n                     WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",\n                )?;\n                statement\n                    .query_map([&row.0], |evidence_row| {\n                        Ok(CoreVoiceTraitEvidenceView {\n                            voice_evidence_id: evidence_row.get(0)?,\n                            current_status: evidence_row.get(1)?,\n                        })\n                    })?\n                    .collect::<Result<Vec<_>, _>>()?\n            };''',
    '''            let evidence = {\n                let mut statement = connection.prepare(\n                    "SELECT ve.voice_evidence_id,ve.status FROM core_voice_trait_evidence cve\n                     JOIN voice_evidence ve ON ve.voice_evidence_id=cve.voice_evidence_id\n                     WHERE cve.trait_id=?1 ORDER BY ve.voice_evidence_id",\n                )?;\n                let rows = statement\n                    .query_map([&row.0], |evidence_row| {\n                        Ok(CoreVoiceTraitEvidenceView {\n                            voice_evidence_id: evidence_row.get(0)?,\n                            current_status: evidence_row.get(1)?,\n                        })\n                    })?\n                    .collect::<Result<Vec<_>, _>>()?;\n                rows\n            };''',
    "Core Voice evidence rows lifetime",
)
write(path, text)

path = "src/components/VoiceWorkspace.tsx"
text = read(path)
text = replace_once(
    text,
    'evidenceIds: trait.evidence.map((item) => item.voiceEvidenceId),',
    'evidenceIds: trait.evidence.filter((item) => item.currentStatus === "eligible").map((item) => item.voiceEvidenceId),',
    "omit invalidated evidence when editing trait",
)
text = replace_once(
    text,
    'onClick={() => updateTraitDraft(voice.voiceId, { ...EMPTY_TRAIT })}',
    'onClick={() => updateTraitDraft(voice.voiceId, { ...EMPTY_TRAIT, traitId: undefined })}',
    "clear trait id on cancel",
)
write(path, text)
