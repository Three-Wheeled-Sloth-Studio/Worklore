# WorkLore

Turn the work you did into stories you can actually use.

WorkLore is a local-first career story bank. It helps uncover useful, evidence-backed stories from resumes and other career sources, then preserves those stories as user-owned records instead of trapping them in an opaque application database.

## Current Development Slice

The current desktop foundation can:

- Create and reopen a local WorkLore vault
- Copy TXT, Markdown, PDF, and DOCX sources into that vault
- Detect duplicate files with SHA-256
- Extract local text from supported documents
- Scan extracted text for private entities and assign stable redaction tokens
- Review and resolve ambiguous entity mappings
- Extract resume bullets into story candidates
- Restrict resume candidate extraction to recognized Work History or Employment History sections
- Ignore headline job titles, career summaries, skills, education, and other non-employment sections
- Record local operation timings for import, extraction, and privacy-scan phases

The application remains an early test build. Provider integrations, polished story synthesis, and broader career-source workflows are still under development.

## Development

Requirements:

- Windows 11
- Node.js 22 or a compatible current release
- Rust stable toolchain
- Microsoft WebView2 runtime

Run the desktop development environment:

```powershell
./dev.bat
```

Development compilation is written outside the repository under the configured WorkLore external root. By default this is:

```text
%LOCALAPPDATA%\WorkLore\build\dev
```

## Validation

Run the source validation build without producing an installer:

```powershell
npm run validate
```

Build output, Rust targets, frontend bundles, and validation logs are written outside the repository. The build scripts fail when known build or install folders appear inside the repository.

## QA Install

Build, stage, and launch the QA runtime:

```powershell
./qa.bat
```

The default QA install location is:

```text
%LOCALAPPDATA%\WorkLore\installed\qa
```

The QA executable is copied into that external install folder before it runs. It is not launched from the repository or from the compiler target directory.

Set `WORKLORE_EXTERNAL_ROOT` to move all external development, build, and QA folders to another location.

## Local Instrumentation

WorkLore records operation timing metadata inside each vault at:

```text
.worklore/operation-metrics/
```

Metrics include operation names, phases, durations, outcomes, file sizes, and character counts. They do not include resume text, interview responses, or provider request bodies.

The desktop workspace shows active phases and the slowest recent operations so long-running local tasks can be inspected without rummaging through logs like a raccoon in a server closet.

## License

WorkLore is licensed under the GNU Affero General Public License v3.0 only, identified as `AGPL-3.0-only`. See `LICENSE` for the full terms and `LICENSE_POLICY.md` for the studio's default public-repository policy and exceptions.
