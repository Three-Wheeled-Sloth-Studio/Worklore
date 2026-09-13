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
