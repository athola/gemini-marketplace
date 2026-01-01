# API & Data Requirements Checklist: Gemini CLI Extension Marketplace

**Purpose**: Author self-check focusing on manifest ingestion, caching, rate limits, and REST/CLI data contracts
**Created**: 2025-10-09
**Feature**: specs/001-build-a-gemini/spec.md

## Requirement Completeness

- [X] **CHK001**: Are the required manifest fields fully enumerated in the requirements?
- [X] **CHK002**: Do the requirements specify all data points returned by the list endpoint/CLI?
- [X] **CHK003**: Is the default curated source documented along with its enable/disable behavior?
- [X] **CHK004**: Are cache metadata expectations fully described?

## Requirement Clarity

- [X] **CHK005**: Is the process for slug generation and namespace format unambiguous?
- [X] **CHK006**: Are error responses for manifest validation failures clearly defined?
- [X] **CHK007**: Are rate-limit countdown behaviors and exposed fields explicitly stated?
- [X] **CHK008**: Is the distinction between registry detection and a secondary filesystem scan for install status spelled out?

## Requirement Consistency

- [X] **CHK009**: Do the CLI display requirements align with the REST schemas in the OpenAPI contract?
- [X] **CHK010**: Are cache invalidation rules consistent between the Functional Requirements and Edge Case handling?
- [X] **CHK011**: Is credential-helper reliance described consistently across the Clarifications, Functional Requirements, and Assumptions?

## Acceptance Criteria Quality

- [X] **CHK012**: Can the acceptance criteria for User Story 1 be objectively evaluated for data fields and grouping behavior?
- [X] **CHK013**: Do the User Story 2 criteria cover every data element promised in the detail view?
- [X] **CHK014**: Are the success metrics traceable to data/API requirements?

## Scenario Coverage

- [X] **CHK015**: Are offline cache browse scenarios documented, including when remote data is unavailable?
- [X] **CHK016**: Are private-source authentication flows and failure messaging specified?
- [X] **CHK017**: Do the requirements address how deleted or relocated repositories affect listings and caching?

## Edge Case Coverage

- [X] **CHK018**: Are malformed or missing `gemini-extension.json` manifests handled consistently across the list and detail views?
- [X] **CHK019**: Is the behavior defined when manifests lack optional fields?

## Non-Functional Requirements

- [X] **CHK020**: Are observability outputs tied to specific events?
- [X] **CHK021**: Are performance targets for cached listing render speed mapped to measurable data operations?
- [X] **CHK022**: Are security expectations around storing no credentials and relying on helpers described for all relevant actions?

## Dependencies & Assumptions

- [X] **CHK023**: Are external GitHub API constraints and metadata format assumptions captured?
- [X] **CHK024**: Does the spec document dependencies on the Gemini CLI extension registry and file layout with assumptions about secondary sources?

## Ambiguities & Conflicts

- [X] **CHK025**: Are there any conflicting statements about enabling/disabling the default source versus user-added sources?
- [X] **CHK026**: Is the terminology for "source", "catalog", and "marketplace" consistent throughout the spec and clarifications?
- [X] **CHK027**: Are data export/import expectations clearly addressed or explicitly marked as out of scope?
