# Windows release signing

WorkLore uses a fail-closed SignPath Foundation release path. Until the open-source application is approved and the repository is configured, Windows preview builds remain explicitly unsigned.

## SignPath Foundation application

Apply at <https://signpath.org/apply> using:

- project: `WorkLore`;
- repository: `https://github.com/Three-Wheeled-Sloth-Studio/Worklore`;
- license: `AGPL-3.0-only`;
- release artifact: portable Windows executable `WorkLore.exe`;
- build system: GitHub Actions on GitHub-hosted Windows runners;
- code signing policy: `CODE_SIGNING_POLICY.md`;
- privacy policy: `PRIVACY.md`; and
- release workflow: `.github/workflows/sign-release.yml`.

Do not claim approval or a SignPath Foundation signature until SignPath confirms the project and an exact release passes Authenticode verification.

## SignPath project configuration

After approval:

1. Install the SignPath GitHub App for this repository.
2. Add the predefined GitHub.com trusted build system to the SignPath organization and WorkLore project.
3. Create a project for WorkLore.
4. Import `.signpath/artifact-configuration.xml` as the artifact configuration.
5. Create a production release-signing policy that requires manual approval and GitHub origin verification.
6. Restrict the trusted build policy to GitHub-hosted runners and the official repository.
7. Create a submitter API token with only the permissions required for the WorkLore release policy.

Configure the GitHub repository with:

| Kind | Name |
| --- | --- |
| Secret | `SIGNPATH_API_TOKEN` |
| Variable | `SIGNPATH_ORGANIZATION_ID` |
| Variable | `SIGNPATH_PROJECT_SLUG` |
| Variable | `SIGNPATH_SIGNING_POLICY_SLUG` |
| Variable | `SIGNPATH_ARTIFACT_CONFIGURATION_SLUG` |

Never commit the API token or copy it into logs, documentation, release notes, or local environment files inside the repository.

## Creating a signed release

1. Update the version consistently in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Complete the normal validation and promotion path through `dev`, `qa`, and `main`.
3. Create and push a `vMAJOR.MINOR.PATCH` tag on the exact `main` commit.
4. Review the GitHub Actions build and the SignPath origin information.
5. Manually approve the signing request in SignPath.
6. Confirm the workflow verifies the signer and timestamp before it creates the GitHub Release.
7. Verify `WorkLore.exe` and `WorkLore.exe.sha256` from the release before updating external download links.

The workflow must never publish the temporary unsigned artifact. Signing changes the executable bytes, so only the checksum produced after successful signing is a release checksum.
