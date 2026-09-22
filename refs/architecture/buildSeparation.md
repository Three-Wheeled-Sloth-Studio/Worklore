# Build And Install Separation

## Decision

The WorkLore repository contains source, tests, fixtures, project references, and configuration only.

Development compiler output, validation output, QA builds, installer staging, logs, and installed runtimes must live outside the repository tree.

## Default External Layout

```text
%LOCALAPPDATA%/WorkLore/
  build/
    dev/
      cargo-target/
      frontend/
      logs/
    validate/
      cargo-target/
      frontend/
      logs/
    qa/
      cargo-target/
      frontend/
      logs/
    release/
      cargo-target/
      frontend/
      logs/
  installed/
    qa/
    release/
```

The root can be changed with `WORKLORE_EXTERNAL_ROOT`.

## Repository Guard

`scripts/assert-repo-clean.ps1` fails when known build or installation folders exist inside the repository, including:

- `dist`
- `build`
- `target`
- `src-tauri/target`
- `qa-install`
- `release`
- `artifacts`

The guard runs before and after local builds and in CI.

## Development

`scripts/dev.ps1` sets `CARGO_TARGET_DIR` to the external development build root before launching Tauri development mode.

The Vite development server remains source-driven and does not produce a repository-local bundle.

## Validation

The validation channel writes frontend output, Cargo targets, and logs to the external validation build root. It does not create an installer.

CI uses the runner temporary directory for the same separation.

## QA

`scripts/install-qa.ps1` performs these steps:

1. Validate that the external build and installation roots are outside the repository.
2. Build the QA runtime using external frontend and Cargo target folders.
3. Copy the QA executable to the external QA installation folder.
4. Write build and installation manifests.
5. Launch the copied QA executable when requested.

QA must never run the binary directly from `src-tauri/target`, an external Cargo target folder, or the repository.

## Channel Identity

QA builds use a distinct product name and application identifier:

```text
Product: WorkLore QA
Identifier: org.threewheeledsloth.worklore.qa
```

Release builds retain the production identity.

This prevents a QA install from quietly sitting on the production application's chair and pretending nothing happened.
