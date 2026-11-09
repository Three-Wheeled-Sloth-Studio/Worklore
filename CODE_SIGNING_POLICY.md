# Code signing policy

## Status

WorkLore is preparing an application to the SignPath Foundation open-source program. Windows builds published before approval are unsigned and are not covered by a SignPath Foundation certificate.

Free code signing provided by [SignPath.io](https://signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

## Project

- Project: WorkLore
- Source: [Three-Wheeled-Sloth-Studio/Worklore](https://github.com/Three-Wheeled-Sloth-Studio/Worklore)
- License: AGPL-3.0-only
- Maintainer: [SlothMD](https://github.com/SlothMD)

The repository contains WorkLore's application source and build definitions. WorkLore is not commercially dual-licensed, and project-owned proprietary components are not included in signed releases.

## Team roles

- Authors, committers, and reviewers: [SlothMD](https://github.com/SlothMD)
- Signing approver: [SlothMD](https://github.com/SlothMD)

Changes from contributors without direct commit access require maintainer review before merge. Everyone assigned a signing role must use multi-factor authentication for GitHub and SignPath.

## Eligible artifacts

Only the production Windows executable named `WorkLore.exe` may be signed under this policy. It must be built from this repository by `.github/workflows/sign-release.yml` on a GitHub-hosted runner from a version tag that points to the protected `main` branch.

Local builds, QA builds, pull-request artifacts, forks, modified distributions, third-party executables, and artifacts produced by other build systems are not eligible. Third-party open-source dependencies may be included in WorkLore, but they are not independently signed as WorkLore-owned binaries.

## Release and approval process

1. The release version is declared consistently in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. A version tag matching that version is created from `main`.
3. GitHub Actions builds and validates the unsigned production executable on a GitHub-hosted Windows runner.
4. The exact unsigned executable is uploaded as a GitHub Actions artifact and submitted through SignPath's GitHub trusted-build connector.
5. The signing approver manually approves every request.
6. The workflow downloads the signed executable and fails unless Authenticode reports a valid SignPath Foundation signature and timestamp.
7. The workflow publishes only the verified signed executable and its SHA-256 checksum to the corresponding GitHub Release.

The release workflow fails closed when SignPath configuration is unavailable or signing or verification fails. It never substitutes an unsigned executable in a signed release.

## Artifact identity

Signing rules must enforce these properties:

- Product name: `WorkLore`
- Company name: `Three-Wheeled Sloth Studio`
- Product and file version: the version supplied by the release tag
- Copyright: `Copyright (c) 2026 Three-Wheeled Sloth Studio`
- Signature algorithm: SHA-256 Authenticode with a trusted timestamp

The reviewed artifact configuration is stored at `.signpath/artifact-configuration.xml`. The active configuration in SignPath must match it.

## Privacy

WorkLore does not transfer information to other networked systems unless the user explicitly requests an operation that requires network access. The [Privacy policy](PRIVACY.md) describes local storage, optional AI-provider requests, and user-triggered public-source discovery.

## Verification

Users can verify a downloaded release in Windows PowerShell:

```powershell
Get-AuthenticodeSignature -LiteralPath ".\WorkLore.exe" |
  Format-List Status,SignerCertificate,TimeStamperCertificate
```

An official signed release must report `Status: Valid`, identify SignPath Foundation in the signer certificate, and include a timestamp certificate. Release checksums provide an additional integrity check but are not a substitute for the Authenticode signature.

## Incident response

Suspected certificate misuse, signing-token compromise, unexpected signed artifacts, or signature verification failures must be reported privately according to `SECURITY.md`. Maintainers will stop signing and publication, preserve relevant audit evidence, rotate or revoke affected credentials, contact SignPath, remove unsafe releases when necessary, and publish corrected release information.
