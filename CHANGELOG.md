# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0.0] - 2026-05-06

### Added
- Complete Tauri v2 desktop app with Rust backend + React/TypeScript frontend
- Smart document editor with Tiptap rich text editing
- Template library with 4D classification (doc attr, business domain, content module, project phase)
- NAS directory scanner for importing templates with auto-categorization
- Deviation checking between bid documents and requirement documents
- Punctuation checking for Chinese document standards
- Self-review engine for local checks (repetition, sensitive info, placeholders)
- AI-powered contradiction and context-logic detection
- Document parser supporting DOCX, PDF, XLSX, TXT, and Markdown
- Meilisearch integration for full-text template search
- SQLite database for local template storage
- Desensitization engine for privacy data masking
- Clipboard export with HTML and plain text formats
- Check results panel with deviation report visualization
- Requirements context provider for cross-tab state management

### Changed
- Parallelized deviation/risk/self-review checks for faster response
- Converted CPU-bound sync commands to async spawn_blocking

### Fixed
- Path traversal vulnerability in document parser (ParentDir detection)
- XSS vulnerability in template preview (DOMPurify sanitization)
- Brace-counting JSON parser for nested objects with escaped quotes
- Unknown severity fallback with proper logging in AI review items
- ID validation for scoring criteria and commitment items

### Infrastructure
- Vitest + React Testing Library test suite
- 71 Rust unit tests + 10 frontend component tests
- CI workflow with automated testing
- agtalk multi-agent collaboration setup
