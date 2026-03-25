---
phase: 06
slug: add-local-mcp-server-for-llm-driven-cli-interaction
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 06 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest 2.x |
| **Config file** | cli/mcp/vitest.config.ts (Wave 0 installs) |
| **Quick run command** | `cd cli/mcp && npx vitest run` |
| **Full suite command** | `cd cli/mcp && npx vitest run --coverage` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd cli/mcp && npx vitest run`
- **After every plan wave:** Run `cd cli/mcp && npx vitest run --coverage`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 1 | D-04 | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "registers analyze_url"` | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 1 | D-05 | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "maps parameters"` | ❌ W0 | ⬜ pending |
| 06-01-03 | 01 | 1 | D-07 | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "passes raw JSON"` | ❌ W0 | ⬜ pending |
| 06-01-04 | 01 | 1 | D-08 | unit | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "returns isError"` | ❌ W0 | ⬜ pending |
| 06-01-05 | 01 | 1 | D-02 | integration | `cd cli/mcp && npx vitest run tests/tool.test.ts -t "stdio"` | ❌ W0 | ⬜ pending |
| 06-01-06 | 01 | 1 | D-03 | unit | `cd cli/mcp && npx vitest run tests/binary.test.ts` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `cli/mcp/tests/tool.test.ts` — stubs for D-04, D-05, D-07, D-08, D-02
- [ ] `cli/mcp/tests/binary.test.ts` — covers D-03 binary resolution
- [ ] `cli/mcp/vitest.config.ts` — test config
- [ ] Framework install: `cd cli/mcp && npm install` — sets up all dev dependencies

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Claude Desktop integration | D-02 | Requires Claude Desktop running locally | Add server to claude_desktop_config.json, verify tool appears in Claude Desktop |
| npx invocation | D-10 | Requires published npm package | After npm publish, run `npx geodaddy-mcp` and verify stdio connects |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
