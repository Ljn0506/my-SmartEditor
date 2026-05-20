# TODOS

## GenerateTab

- ~~**Priority:** P1 — Add useMemo for derived state (selectedCard, filter categories) to reduce re-renders~~ **Completed:** v0.3.1.0 (2026-05-15)
- ~~**Priority:** P2 — Parallelize card generation with Promise.all instead of sequential await~~ **Completed:** v0.3.2.0 (2026-05-18) — 已实现，无需修改
- ~~**Priority:** P2 — Extract STATUS_META / SEVERITY_META into shared constants module~~ **Completed:** v0.3.2.0 (2026-05-18)

## Backend

- ~~**Priority:** P1 — Extract `validate_path` into shared utility (currently duplicated in 3 files)~~ **Completed:** v0.3.1.0 (2026-05-15)
- ~~**Priority:** P2 — Replace O(n*m) deviation check with HashMap-based paragraph lookup~~ **Completed:** v0.4.0.1 (2026-05-20)
- ~~**Priority:** P2 — Optimize self-review similarity calculation (currently O(n²) char-level LCS)~~ **Completed:** v0.4.0.1 (2026-05-20)
- ~~**Priority:** P2 — Add offset pagination to template search (currently capped at 1000)~~ **Completed:** v0.4.0.1 (2026-05-20)
- ~~**Priority:** P3 — Feature-gate heavy document parsing crates (docx-rs, pdf-extract, calamine)~~ **Completed:** v0.4.0.1 (2026-05-20)

## Security

- ~~**Priority:** P1 — Harden SSRF validation: block 127.0.0.0/8, 0.0.0.0, and DNS rebinding~~ **Completed:** v0.3.1.0 (2026-05-15)
- ~~**Priority:** P2 — Sanitize HTML before clipboard export~~ **Completed:** v0.3.1.0 (2026-05-15)
- ~~**Priority:** P2 — Scope path validation to allowed root directory (not just ParentDir check)~~ **Completed:** v0.3.2.0 (2026-05-18)

## Testing

- ~~**Priority:** P1 — Add negative-path tests for `validate_ai_url`~~ **Completed:** v0.3.1.0 (2026-05-15)
- ~~**Priority:** P1 — Add tests for `extract_param_placeholders` and `extract_risk_flags`~~ **Completed:** v0.3.1.0 (2026-05-15)
- ~~**Priority:** P2 — Add tests for `AiClient::chat` timeout and error handling (mock HTTP)~~ **Completed:** v0.3.2.0 (2026-05-18)
- ~~**Priority:** P2 — Use tempfile crate in apply_self_review_fixes tests instead of hardcoded /tmp paths~~ **Completed:** v0.3.2.0 (2026-05-18)

## Completed

- **Priority:** P0 — Phase 3: Multi-file consistency + business card mandatory review **Completed:** v0.3.0.0 (2026-05-11)
- **Priority:** P0 — Phase 4: Remove LibraryTab, rename push→generate, streamline tabs **Completed:** v0.3.0.0 (2026-05-11)
- **Priority:** P0 — Fix /review blockers: path traversal, AI timeout, overwrite backup **Completed:** v0.3.0.0 (2026-05-11)
- **Priority:** P1 — useMemo optimization + SSRF hardening + negative-path tests + cleanup **Completed:** v0.3.1.0 (2026-05-15)
- **Priority:** P1 — NAS scanner v1.0 classification alignment (path inference + filename prefix + security level) **Completed:** v0.3.2.0 (2026-05-18)
- **Priority:** P1 — Security hardening: `validate_path_within` scoped to allowed root directory **Completed:** v0.3.2.0 (2026-05-18)
- **Priority:** P1 — Frontend constants extraction (STATUS_META / SEVERITY_META) **Completed:** v0.3.2.0 (2026-05-18)
- **Priority:** P1 — Testing: AI mock HTTP tests (wiremock) + tempfile tests **Completed:** v0.3.2.0 (2026-05-18)
- **Priority:** P0 — Ship v0.4.0.0 (Phase 1~4 rebuild, smart generation, security hardening) **Completed:** v0.4.0.0 (2026-05-20)
