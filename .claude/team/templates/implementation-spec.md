# Implementation spec: <title> (#<issue>)

Status: draft | approved by <human> on <date>
Links: story #<issue>, design spec, ADRs

## Scope
In and out, including scope negotiated with the PM and Designer.

## Design of the change
Modules and crates touched, new messages (events, requests, streams), state ownership (backend or frontend), data flow. Reference `docs/architecture.md` rules and list any deviation with its ADR.

## Work packages
| WP | Owner | Files and crates (disjoint between packages) | Depends on | Done when |
|---|---|---|---|---|
| WP1 | developer-1 | | | |

## Interfaces
Signatures, message types and payloads that packages share. Fixed before work starts.

## Test strategy
Behaviours to cover as given/when/then, at which level (unit, module, e2e), fakes needed, hardware checks.

## Risks and hardware limits
Reference docs/prd.md section 5.2 and the driver's protocol.md.

## Dependencies
New external dependencies with justification (what, why not write it, licence, maintenance).

## Rollout and migration
