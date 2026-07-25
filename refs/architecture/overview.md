# Architecture Overview

WorkLore is a local-first desktop application. A user-selected vault contains imported source documents, canonical structured records, readable story files, privacy mappings, and exports. A local SQLite database provides rebuildable indexing and search rather than acting as the sole source of truth.

The desktop shell is planned around Tauri 2, React, TypeScript, and Rust. Application workflow state remains deterministic. LLM providers perform bounded operations through structured contracts instead of controlling navigation, persistence, deletion, or workflow state.

Initial provider paths are local Ollama, Gemini using a user-supplied API key, and manual workspace packages for ChatGPT, Claude, or Gemini subscriptions. Provider adapters must share a narrow contract so a later hosted WorkLore provider can be added without changing the core vault or story model.

Each vault contains a durable Private Entity Registry. New and materially changed content is scanned for employers, clients, projects, systems, repositories, people, and other private entities. Known aliases are reused, ambiguous matches are reviewed by the user, and stable tokens support redacted cloud requests and anonymized public output.

Local Git scanning reads registered checkouts and discovered repositories beneath a selected parent folder. GitHub account access, private repository OAuth, live synchronization, hosted accounts, and billing are later boundaries.
