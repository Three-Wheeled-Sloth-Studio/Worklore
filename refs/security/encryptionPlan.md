# Vault Encryption Plan

## Decision

Vault-level encryption at rest will follow the first technical workflow test rather than block it.

Early testers should use operating-system disk encryption, such as BitLocker, while WorkLore validates the core vault, source, privacy, interview, and recovery workflows.

## Why It Is Deferred

Adding vault encryption changes more than file writing. It affects:

- Key creation and storage
- Recovery when a key or machine is lost
- Backups and restore
- Search and indexing
- Temporary extraction files
- Provider workspace exports
- Migrations
- Crash recovery
- Multi-machine vault use
- Diagnostic support

The first test should expose the actual access patterns before those patterns are sealed inside a cryptographic container that is extremely secure and slightly on fire.

## First-Test Guardrails

Before vault encryption is implemented:

- Keep vaults on encrypted operating-system volumes
- Do not store provider credentials in the vault
- Do not retain provider request or response bodies by default
- Keep temporary extracted text inside the vault's rebuildable `.worklore` directory
- Avoid cloud-synchronized vaults unless the user understands the provider's storage model
- Back up the vault through an encrypted destination
- Treat local access to the user account as access to the vault

## Encryption Increment

The hardening increment should begin after the first five-person technical test and before broad public distribution.

It should include:

1. A threat model covering device loss, local account compromise, backups, sync providers, and support diagnostics.
2. Per-vault encryption keys generated locally.
3. Key wrapping through the operating-system credential store.
4. A recovery-key workflow that is optional but strongly encouraged.
5. Authenticated encryption for canonical records and copied source files.
6. Encrypted or memory-only handling for extraction caches and indexes.
7. Atomic encrypted writes and migration recovery.
8. Backup and restore tests across the primary workstation and backup laptop.
9. Clear failure states that never replace unreadable encrypted data with empty records.
10. Performance instrumentation for encryption, decryption, indexing, backup, and restore phases.

## Open Design Questions For That Increment

- Encrypt individual files or package the vault into an encrypted container
- How to support portable vaults across multiple Windows machines
- Whether source files should remain separately accessible outside WorkLore
- How recovery keys are exported and verified
- Whether cloud-sync conflict behavior is acceptable for encrypted records
- Whether an optional read-only recovery tool should be distributed separately

No implementation should claim the vault is encrypted until copied sources, canonical records, caches, indexes, backups, and recovery journals have all been covered or explicitly excluded.
