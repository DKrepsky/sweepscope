# SweepScope team workflow

Every agent reads this file at the start of a task, then its own memory file, then its inbox.

## 1. Roles

| Agent | Owns | Main output | Never |
|---|---|---|---|
| `product-manager` | Requests, requirements, user stories, bugs, release confirmation | GitHub issues | writes code or designs |
| `designer` | UX, workflows, design system, tokens, component specs | Design spec (+ wireframes) | writes production code |
| `staff-engineer` | Architecture, ADRs, planning, scope negotiation | Implementation spec, ADRs | writes production code |
| `developer` | Implementation and its tests | Source code + tests | changes specs or architecture |
| `qa` | Quality | Test plan, e2e tests, manual checks, QA report | fixes product code |
| `code-reviewer` | Code quality, bugs, security, architecture guard | Review findings | edits code |
| `product-manager` also acts as **orchestrator** | The main session agent: the human talks only to the PM, who runs this workflow, spawns the other agents and relays approvals | | decides on behalf of the human |

The human approves at four gates. No agent skips a gate.

## 2. Flow

```
 human ─▶ PM ─▶ [G1 human: story] ─▶ Designer* ─▶ [G2 human: design] ─▶ Staff Engineer ─▶ [G3 human: implementation spec]
        ─▶ Developers ─▶ QA ─▶ Code Reviewer ─▶ PM ─▶ [G4 human: confirm feature] ─▶ push
                          ▲  bugs ──┘   ▲  review issues ──┘ (back to Developers)
 * only when the story has UI or UX impact. The PM states "design: needed / not needed" in the story.
```

| Step | Agent | Does | Ends with |
|---|---|---|---|
| 1 | PM | Takes the request, asks the human clarifying questions | Questions or a draft |
| 2 | PM | Creates the GitHub issue (story with acceptance criteria, or bug) | Issue URL, status `NEEDS_HUMAN_APPROVAL` (**G1**) |
| 3 | PM | After approval: hands off to Designer (or straight to Staff Engineer if no design is needed) | `handoff` message |
| 4 | Designer | Journey, workflow, UI, tokens and components; wireframes when layout or flow is new | Design spec committed under `docs/specs/`, status `NEEDS_HUMAN_APPROVAL` (**G2**) |
| 5 | Designer | After approval: hands off to Staff Engineer | `handoff` message |
| 6 | Staff Engineer | Clarifies with PM and Designer through the bus, reports problems, negotiates scope, writes ADRs when a decision is architectural, writes the implementation spec with work packages | Implementation spec, status `NEEDS_HUMAN_APPROVAL` (**G3**) |
| 7 | Developers | One developer per work package, in parallel when files are disjoint; implement and test (BDD) | `handoff` to QA with what to verify |
| 8 | QA | Test plan first, then automated e2e and manual and UI verification; QA report | Report to PM and Staff; `bug` messages to developers |
| 9 | Code Reviewer | Reviews the diff against the architecture rules and guarantees every non-test CI check passes (lint, format, `cargo deny`, `cargo audit`, npm audit, architecture) | `feedback` to developers or an approval report with the CI parity table |
| 10 | PM | Presents the finished feature against the acceptance criteria | Status `NEEDS_HUMAN_APPROVAL` (**G4**) |
| 11 | PM | After G4 only: push, following section 6 | |

Loops:

- QA finds a bug → `bug` message to the developer → fix → QA retests the affected cases.
- The Code Reviewer re-runs the CI parity gate on the final state before G4 (new security advisories appear without code changes); a red check is a blocker and never an exception.
- Review finds issues → `feedback` to the developer → fix → review of the changed parts. If a fix changes behaviour, QA retests.
- More than three rounds on the same problem → the PM stops and asks the Staff Engineer, then the human.
- A change to the story or the design during planning goes back to the PM or Designer and through the gate again. Scope cuts that remove an acceptance criterion need the human's approval (G1 again).

## 3. Artefacts

| Artefact | Location | Committed |
|---|---|---|
| Story, bug | GitHub issue from `.github/ISSUE_TEMPLATE/` (labels `enhancement` or `bug`) | n/a |
| Design spec, wireframes | `docs/specs/<issue>-<slug>/design-spec.md`, `prototypes/` | yes |
| Implementation spec | `docs/specs/<issue>-<slug>/implementation-spec.md` | yes |
| ADR | `docs/adr/NNNN-<slug>.md` | yes |
| Test plan, QA report | `docs/specs/<issue>-<slug>/test-plan.md`, `qa-report.md` | yes |
| Design tokens and component specs | `docs/design-system/` | yes |
| Source code and tests | repository | yes |
| Messages | `.claude/bus/` | **never** |
| Screenshots, logs, scratch files | `.claude/tmp/<issue>/` | **never** |
| Agent memory | `.claude/memory/<agent>.md` | yes (durable lessons only, no secrets) |

`<slug>` is the issue title in kebab-case. Templates are in `.claude/team/templates/`.

## 4. Message bus

Agents talk through `.claude/scripts/bus.sh`. Agents are short-lived, so the bus is the record and the PM (main session) is the delivery mechanism.

```sh
.claude/scripts/bus.sh inbox <me>                         # at the start of every task
.claude/scripts/bus.sh read <me> <id>
.claude/scripts/bus.sh ack <me> <id>                      # once handled
.claude/scripts/bus.sh send --from <me> --to <agent> --issue <n> --type <type> --subject "<one line>" <<'MSG'
body
MSG
```

Types: `handoff`, `question`, `answer`, `feedback`, `bug`, `approval-request`, `approved`, `rejected`, `report`.

Rules:

- Every handoff names the issue, the artefact paths, what is expected next and what is still open.
- Questions go to the owner of the answer: product questions to the PM, UX questions to the Designer, technical ones to the Staff Engineer.
- Questions for the human are never sent on the bus. They go in the agent's final reply with status `NEEDS_INFO` and the PM asks the human.
- Developers are addressed as `developer-1`, `developer-2` … when work is split.
- An agent finishes by replying to the PM with: status (`DONE`, `NEEDS_HUMAN_APPROVAL`, `NEEDS_INFO`, `BLOCKED`), the artefact paths, the messages sent, and the next agent expected.

## 5. Memory

Each agent keeps `.claude/memory/<agent>.md`: read it first, update it last. Keep only durable, reusable knowledge: decisions, conventions, mistakes not to repeat, user preferences. No task logs, no secrets, no personal data. Keep the file short and rewrite it rather than appending forever.

## 6. Repository rules

- Work on a feature branch named `feat/<issue>-<slug>` or `fix/<issue>-<slug>`. `main` is protected: changes go through a pull request that the human merges.
- Spikes (ADR 0008): code on `spike/<issue>-<slug>`, never merged, tagged `spike-<issue>-final`; the report and raw results go to `main` from `docs/spike-<issue>-report`, touching only `docs/spikes/`.
- Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `chore:`), with the issue number in the body.
- Commit and push only after G4, except that developers may commit locally on the feature branch.
- Anything that publishes (releases, crates, packages) or changes repository security settings needs explicit human confirmation.
- Spec files are committed together with the code of the feature they describe.
- Source of truth for design rules: `docs/architecture.md`. For requirements and hardware limits: `docs/prd.md`. For the device protocol: the driver repo's `docs/protocol.md` (`/home/kr/workspace/hantek6xx4-lib`).

## 7. Definition of done

A feature is done when: the acceptance criteria pass; unit, integration and e2e tests exist in given/when/then style and pass; formatting, lint, architecture and dependency security checks (`cargo deny`, `cargo audit`, `npm audit`) pass, as verified by the Code Reviewer on the final branch state (never lowered or ignored to get green); QA reported no open bugs; the review has no open findings; specs and ADRs are current; the human confirmed it at G4.
