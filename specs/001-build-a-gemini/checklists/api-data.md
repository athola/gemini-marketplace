# API & Data Requirements Checklist: Gemini CLI Extension Marketplace

**Purpose**: Author self-check focusing on manifest ingestion, caching, rate limits, and REST/CLI data contracts
**Created**: 2025-10-09
**Feature**: specs/001-build-a-gemini/spec.md

## Requirement Completeness

- [X] CHK001 Are the required manifest fields (name, version, author, repository, categories, compatibility) fully enumerated in requirements? [Completeness, Spec §Functional Requirements FR-013a]
- [X] CHK002 Do requirements specify all data points returned by the list endpoint/CLI (display name, namespace, version, install status, source)? [Completeness, Spec §Functional Requirements FR-002]
- [X] CHK003 Is the default curated source (`https://github.com/athola/gemini-marketplace`) documented along with enable/disable behavior? [Completeness, Spec §Functional Requirements FR-005–FR-005a]
- [X] CHK004 Are cache metadata expectations (TTL, manual refresh, storage path) fully described? [Completeness, Spec §Functional Requirements FR-010–FR-011]

## Requirement Clarity

- [X] CHK005 Is the process for slug generation and namespace format unambiguous? [Clarity, Spec §Functional Requirements FR-002a]
- [X] CHK006 Are error responses for manifest validation failures clearly defined (fields, messaging)? [Clarity, Spec §Clarifications 2025-10-09, Spec §Edge Cases]
- [X] CHK007 Are rate-limit countdown behaviors and exposed fields (reset time, remaining) explicitly stated? [Clarity, Spec §Functional Requirements FR-012a]
- [X] CHK008 Is the distinction between registry detection versus a secondary filesystem scan for install status spelled out? [Clarity, Spec §Functional Requirements FR-014]

## Requirement Consistency

- [X] CHK009 Do the CLI display requirements align with REST schemas in the OpenAPI contract (field names/structures)? [Consistency, Spec §Functional Requirements FR-001–FR-004, Contracts/marketplace-openapi.yaml]
- [X] CHK010 Are cache invalidation rules consistent between Functional Requirements and Edge Case handling? [Consistency, Spec §Functional Requirements FR-010–FR-011, Spec §Edge Cases]
- [X] CHK011 Is credential-helper reliance described consistently across Clarifications, Functional Requirements, and assumptions? [Consistency, Spec §Clarifications 2025-10-09, Spec §Functional Requirements FR-013c, Spec §Dependencies]

## Acceptance Criteria Quality

- [X] CHK012 Can the acceptance criteria for User Story 1 be objectively evaluated for data fields and grouping behavior? [Acceptance Criteria, Spec §User Story 1]
- [X] CHK013 Do User Story 2 criteria cover every data element promised in the detail view (README excerpt, compatibility, install instructions)? [Acceptance Criteria, Spec §User Story 2]
- [X] CHK014 Are success metrics SC-001 through SC-007 traceable to data/API requirements (e.g., cache latency ties to FR-010)? [Acceptance Criteria, Spec §Success Criteria]

## Scenario Coverage

- [X] CHK015 Are offline cache browse scenarios documented, including when remote data is unavailable? [Coverage, Spec §Assumptions, Spec §Functional Requirements FR-010]
- [X] CHK016 Are private-source authentication flows and failure messaging specified? [Coverage, Spec §Clarifications 2025-10-09, Spec §Functional Requirements FR-013c]
- [X] CHK017 Do requirements address how deleted or relocated repositories affect listings and caching? [Coverage, Spec §Edge Cases]

## Edge Case Coverage

- [X] CHK018 Are malformed or missing `gemini-extension.json` manifests handled consistently across list and detail views? [Edge Case, Spec §Clarifications 2025-10-09, Spec §Edge Cases]
- [X] CHK019 Is behavior defined when manifests lack optional fields (tags, documentation, compatibility)? [Edge Case, Spec §Functional Requirements FR-013a]

## Non-Functional Requirements

- [X] CHK020 Are observability outputs (dual-mode logs, structured metrics) tied to specific events (cache hit/miss, rate-limit queue)? [Non-Functional, Spec §Clarifications 2025-10-09, Spec §Non-Functional Requirements NFR-001]
- [X] CHK021 Are performance targets for cached listing render speed mapped to measurable data operations? [Non-Functional, Spec §Success Criteria SC-003]
- [X] CHK022 Are security expectations around storing no credentials and relying on helpers described for all relevant actions? [Non-Functional, Spec §Functional Requirements FR-013c]

## Dependencies & Assumptions

- [X] CHK023 Are external GitHub API constraints and metadata format assumptions captured, including rate-limit policies? [Dependencies, Spec §Dependencies]
- [X] CHK024 Does the spec document dependencies on Gemini CLI extension registry/file layout with assumptions about secondary sources? [Dependencies, Spec §Dependencies, Spec §Functional Requirements FR-014]

## Ambiguities & Conflicts

- [X] CHK025 Are there any conflicting statements about enabling/disabling the default source versus user-added sources? [Ambiguity, Spec §Functional Requirements FR-005–FR-007]
- [X] CHK026 Is terminology for “source”, “catalog”, and “marketplace” consistent throughout the spec and clarifications? [Consistency, Spec §Clarifications 2025-10-09, Spec §Functional Requirements]
- [X] CHK027 Are data export/import expectations (e.g., manifest schema versioning) clearly addressed or explicitly marked out of scope? [Gap, Spec §Dependencies, Spec §Out of Scope]
