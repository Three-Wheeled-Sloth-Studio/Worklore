from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "src-tauri/src/services/voice_analysis_service.rs"

text = PATH.read_text(encoding="utf-8")

old_call = '''    let _ = audit_provider_run(
        vault_path,
        &run_id,
        &provider_id,
        request.model_id.trim(),
        selected_ids.len(),
        proposal_count,
        outcome,
        error_code,
    );'''
new_call = '''    let _ = audit_provider_run(
        vault_path,
        ProviderRunAudit {
            run_id: &run_id,
            provider_id: &provider_id,
            model_id: request.model_id.trim(),
            evidence_count: selected_ids.len(),
            proposal_count,
            outcome,
            error_code,
        },
    );'''
if text.count(old_call) != 1:
    raise RuntimeError(f"provider audit call: expected 1 match, found {text.count(old_call)}")
text = text.replace(old_call, new_call, 1)

old_function = '''fn audit_provider_run(
    vault_path: &Path,
    run_id: &str,
    provider_id: &str,
    model_id: &str,
    evidence_count: usize,
    proposal_count: usize,
    outcome: &str,
    error_code: Option<&str>,
) -> ServiceResult<()> {
    let connection = Connection::open(vault_path.join(canonical_store::DATABASE_RELATIVE_PATH))?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)\\n         VALUES (?1,'provider_operation','provider_run',?2,'system',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            run_id,
            json!({
                "providerId": provider_id,
                "modelId": model_id,
                "operationId": ANALYZE_VOICE_EVIDENCE_OPERATION,
                "operationVersion": ANALYZE_VOICE_EVIDENCE_VERSION,
                "outcome": outcome,
                "evidenceCount": evidence_count,
                "proposalCount": proposal_count,
                "errorCode": error_code
            })
            .to_string(),
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}'''
new_function = '''struct ProviderRunAudit<'a> {
    run_id: &'a str,
    provider_id: &'a str,
    model_id: &'a str,
    evidence_count: usize,
    proposal_count: usize,
    outcome: &'a str,
    error_code: Option<&'a str>,
}

fn audit_provider_run(vault_path: &Path, audit: ProviderRunAudit<'_>) -> ServiceResult<()> {
    let connection = Connection::open(vault_path.join(canonical_store::DATABASE_RELATIVE_PATH))?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute(
        "INSERT INTO audit_events(audit_id,event_type,record_type,record_id,actor,details_json,occurred_at)\\n         VALUES (?1,'provider_operation','provider_run',?2,'system',?3,?4)",
        params![
            format!("audit_{}", Uuid::now_v7()),
            audit.run_id,
            json!({
                "providerId": audit.provider_id,
                "modelId": audit.model_id,
                "operationId": ANALYZE_VOICE_EVIDENCE_OPERATION,
                "operationVersion": ANALYZE_VOICE_EVIDENCE_VERSION,
                "outcome": audit.outcome,
                "evidenceCount": audit.evidence_count,
                "proposalCount": audit.proposal_count,
                "errorCode": audit.error_code
            })
            .to_string(),
            Utc::now().to_rfc3339()
        ],
    )?;
    Ok(())
}'''
if text.count(old_function) != 1:
    raise RuntimeError(f"provider audit function: expected 1 match, found {text.count(old_function)}")
text = text.replace(old_function, new_function, 1)

old_test_call = '''        audit_provider_run(
            &path,
            "provider_run_test",
            "ollama",
            "qwen3",
            2,
            1,
            "succeeded",
            None,
        )
        .unwrap();'''
new_test_call = '''        audit_provider_run(
            &path,
            ProviderRunAudit {
                run_id: "provider_run_test",
                provider_id: "ollama",
                model_id: "qwen3",
                evidence_count: 2,
                proposal_count: 1,
                outcome: "succeeded",
                error_code: None,
            },
        )
        .unwrap();'''
if text.count(old_test_call) != 1:
    raise RuntimeError(f"provider audit test call: expected 1 match, found {text.count(old_test_call)}")
text = text.replace(old_test_call, new_test_call, 1)

PATH.write_text(text, encoding="utf-8", newline="\n")
