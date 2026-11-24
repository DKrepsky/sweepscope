<!-- Body of a `feedback` or `report` message from the code reviewer -->
## Verdict
approved | changes requested

## Findings
| # | Severity (blocker, major, minor, nit) | File:line | Problem | Required change |
|---|---|---|---|---|

## CI parity (every non-test check of .github/workflows/ci.yml, run on the final branch)
| Check | Command | Result |
|---|---|---|
| format, clippy, cargo deny, cargo audit, generated types | | |
| frontend lint, lint:css, format:check, typecheck, build, npm audit | | |
| architecture, test names, tools tests, actionlint, root npm audit | | |

Any failure is a blocker; none was ignored or silenced.

## Architecture check
Layering, module decoupling, state ownership, dependency rules, tests in given/when/then, comments and naming.

## Security check

## Notes for next time
