# Phase 3: GEO Differentiators - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 03-geo-differentiators
**Areas discussed:** Scoring integration, Listicle detection, AI bot audit, Schema stacking

---

## Scoring Integration

| Option | Description | Selected |
|--------|-------------|----------|
| Equal thirds (Recommended) | Overall = (tech + content + geo) / 3. Simple, consistent. | ✓ |
| GEO weighted higher | GEO gets 40%, tech/content 30% each. | |
| You decide | Claude picks weighting. | |

**User's choice:** Equal thirds
**Notes:** Consistent with current tech/content averaging approach.

---

| Option | Description | Selected |
|--------|-------------|----------|
| All critical (10pts each) | All GEO checks are critical severity. | |
| Mixed severity (Recommended) | AI bot = critical (10pts), listicle = warning (5pts), schema = warning (5pts). | ✓ |
| You decide | Claude assigns based on research. | |

**User's choice:** Mixed severity
**Notes:** Bot blocking is most actionable; listicle/schema are more advisory.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Always present (Recommended) | CategoryScores always includes geo field. | ✓ |
| Only when relevant | Omit geo field if no issues. | |

**User's choice:** Always present
**Notes:** Consistent JSON shape for CI/CD consumers.

---

## Listicle Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Broad detection (Recommended) | Top N headings, ordered lists, numbered headings, comparison tables. | ✓ |
| Strict detection | Only explicit "Top N" title patterns and <ol> lists. | |
| You decide | Claude determines breadth. | |

**User's choice:** Broad detection
**Notes:** Cast a wide net — listicle format is broadly beneficial for AI citation.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Warn with suggestion (Recommended) | Warn when no listicle found, suggest restructuring. | ✓ |
| Info only | Pass with informational message, don't penalize. | |
| You decide | Claude determines status. | |

**User's choice:** Warn with suggestion
**Notes:** None.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, specific type (Recommended) | Pass message includes what type of listicle was found. | ✓ |
| Simple pass | Just "Listicle format detected" with no breakdown. | |

**User's choice:** Yes, specific type
**Notes:** Helps user understand what's working.

---

## AI Bot Audit

| Option | Description | Selected |
|--------|-------------|----------|
| Big 3 (Recommended) | GPTBot, ClaudeBot, PerplexityBot. | |
| Extended list | Big 3 plus GoogleOther, Bytespider, CCBot. | ✓ |
| You decide | Claude picks based on AI landscape. | |

**User's choice:** Extended list
**Notes:** User wants broader coverage with 6 bots.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Warn with details (Recommended) | Single combined result listing blocked/allowed bots. | |
| Fail if any blocked | Single fail result if any bot blocked. | |
| Per-bot results | One AnalysisResult per bot (6 results). | ✓ |

**User's choice:** Per-bot results
**Notes:** Most granular approach. 6 individual results instead of 1 combined.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Fail per blocked bot (Recommended) | Each blocked bot = fail status. | ✓ |
| Warn per blocked bot | Each blocked bot = warn status. | |
| You decide | Claude assigns per bot relevance. | |

**User's choice:** Fail per blocked bot
**Notes:** Direct and actionable per-bot fail status.

---

## Schema Stacking

| Option | Description | Selected |
|--------|-------------|----------|
| Report partial too (Recommended) | Pass=all 3, Warn=1-2, Fail=none. | ✓ |
| Triple only | Pass=all 3, Fail=anything less. Binary check. | |
| You decide | Claude determines granularity. | |

**User's choice:** Report partial too
**Notes:** Gives actionable guidance on which schema types to add.

---

| Option | Description | Selected |
|--------|-------------|----------|
| JSON-LD only (Recommended) | Only parse script type=application/ld+json. | ✓ |
| All formats | Also scan Microdata and RDFa. | |

**User's choice:** JSON-LD only
**Notes:** Consistent with CONT-02 which already parses JSON-LD.

---

## Claude's Discretion

- Exact regex patterns for listicle heading detection
- robots.txt parsing edge cases for AI bot detection
- Schema type matching logic in JSON-LD
- Comparison table detection heuristics

## Deferred Ideas

None — discussion stayed within phase scope.
