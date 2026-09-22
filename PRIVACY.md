# Privacy policy

WorkLore is a local-first desktop application for developing evidence-backed professional stories and written material.

## Local data

WorkLore stores imported documents, extracted text, career stories, proof points, drafts, preferences, privacy mappings, and operation metadata in user-selected local vaults. Three-Wheeled Sloth Studio does not operate a WorkLore account service, synchronization service, telemetry service, or hosted storage backend.

## Network access

WorkLore does not transfer information to other networked systems unless the user explicitly requests an operation that requires network access.

User-triggered network operations can include:

- sending a bounded request to a provider explicitly selected and configured by the user, such as OpenAI or Gemini;
- communicating with a user-configured local Ollama service;
- retrieving public sources when the user runs timely-topic discovery; and
- opening links or other external resources selected by the user.

Provider-assisted requests use the provider and credentials selected by the user. WorkLore applies its configured privacy and redaction controls before eligible cloud requests, but users remain responsible for reviewing the information they choose to send and the selected provider's privacy terms.

WorkLore does not perform automatic publishing, background discovery monitoring, or undisclosed telemetry.

## User control

Users choose where vaults are stored and may back up, move, export, or delete them with ordinary filesystem tools. Removing the WorkLore executable does not delete user-created vaults.

## Security reports

Report suspected vulnerabilities using GitHub private vulnerability reporting as described in `SECURITY.md`. Do not include private vault contents or credentials in a public issue.
