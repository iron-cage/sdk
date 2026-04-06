# Task 026: Documentation and examples

## Goal
Create all public-facing documentation needed for open-source adoption: a README with a working quick-start guide, runnable example programs covering core use cases, an architecture diagram, a contributing guide, and a CHANGELOG. Users must be able to go from `pip install iron-cage` to a running example without consulting internal documentation.

## Dependencies
- Task 019

## In Scope
- README rewrite with quick-start guide matching the presentation flow
- Examples directory with runnable programs (basic usage, LangChain integration, budget enforcement, PII detection)
- Architecture diagram derived from presentation materials
- Contributing guide (build, test, PR process)
- CHANGELOG generated from git history

## Out of Scope
- API reference documentation (covered by rustdoc in Task 024)
- Video tutorials or screencasts
- Translated documentation in non-English languages
- Hosted documentation site (e.g., mdBook or Docusaurus deployment)

## Description
Iron-cage has over 90 internal documentation files, but none of them are oriented toward an external user encountering the project for the first time. The README needs a rewrite that leads with the value proposition, provides a quick-start guide matching the presentation flow, and links to deeper resources.

The examples directory should contain self-contained, runnable programs demonstrating core use cases: basic agent setup, LangChain integration, budget enforcement, and PII detection. Each example should include comments explaining what it does and how to run it. An architecture diagram, derived from the existing presentation materials, gives users a visual understanding of the system. A contributing guide lowers the barrier for external contributors by documenting the build, test, and PR process. A CHANGELOG generated from the git history provides release-level visibility into project evolution.

## Context
Internal documentation is excellent (90+ docs). Public-facing documentation (README, examples, quick start) is needed for open source adoption.

Critical areas:
- `readme.md`
- `docs/`
- Examples directory (to be created)

## Work Procedure
1. Draft the README structure: badges, one-line description, value proposition, quick-start, architecture overview, links.
2. Write the quick-start section with copy-pasteable commands that work after `pip install iron-cage`.
3. Create the `examples/` directory at the repository root.
4. Write `examples/basic_usage.py` demonstrating minimal agent setup and a single LLM call.
5. Write `examples/langchain_integration.py` showing iron-cage wrapping a LangChain agent.
6. Write `examples/budget_enforcement.py` demonstrating budget limits and enforcement behavior.
7. Write `examples/pii_detection.py` demonstrating PII detection and redaction.
8. Create the architecture diagram (SVG or PNG) from presentation materials and embed it in the README.
9. Write `CONTRIBUTING.md` covering: prerequisites, build instructions, test commands, PR process.
10. Generate `CHANGELOG.md` from git history using conventional commit parsing.

## Implementation plan
1. Write README with quick start matching presentation flow.
2. Create examples directory: basic usage, LangChain integration, budget enforcement, PII detection.
3. Add architecture diagram (from presentation material).
4. Write contributing guide.
5. Generate CHANGELOG from git history.

## Test Matrix
| Input/Scenario | Expected Behavior | Pass Criteria |
|---|---|---|
| Fresh machine with `pip install iron-cage` | Quick-start commands succeed | User can run first example without errors |
| `python examples/basic_usage.py` | Agent makes LLM call and prints result | Script exits 0 with output |
| `python examples/langchain_integration.py` | LangChain agent runs through iron-cage | Script exits 0 with output |
| `python examples/budget_enforcement.py` | Budget limit enforced, over-budget call rejected | Script demonstrates enforcement |
| `python examples/pii_detection.py` | PII detected and redacted in output | Script shows redacted text |
| Follow CONTRIBUTING.md build steps | Project builds from source | `cargo build --workspace` succeeds |

## Validation Checklist
- [ ] README contains quick-start guide with copy-pasteable commands
- [ ] README includes architecture diagram
- [ ] All example scripts run without errors
- [ ] Each example includes comments explaining what it does
- [ ] CONTRIBUTING.md covers prerequisites, build, test, and PR process
- [ ] CHANGELOG.md covers all releases from git history
- [ ] No broken links in README or CONTRIBUTING.md

## Validation Procedure
1. On a fresh Python virtual environment, run `pip install iron-cage` and follow the quick-start steps in the README.
2. Run each example script and verify it completes without errors.
3. Follow the CONTRIBUTING.md build instructions from a clean checkout and verify the project builds.
4. Review the CHANGELOG.md for completeness against git tags.
5. Check all links in README and CONTRIBUTING.md for validity (no 404s).
6. Have a team member unfamiliar with the project follow the README and provide feedback.

## Acceptance Criteria
- README quick start works on a fresh machine with `pip install iron-cage`.
- Each example runs successfully.
- Contributing guide covers: build, test, PR process.
- CHANGELOG covers all releases.
