#!/usr/bin/env bash
# File-based message bus for the SweepScope agent team. Messages are local only (see .gitignore).
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
bus="$root/.claude/bus"
agent_pattern='^(product-manager|designer|staff-engineer|developer(-[0-9]+)?|qa|code-reviewer)$'
type_pattern='^(handoff|question|answer|feedback|bug|approval-request|approved|rejected|report)$'

fail() { echo "bus: $*" >&2; exit 1; }

valid_agent() { [[ "$1" =~ $agent_pattern ]] || fail "unknown agent '$1'"; }

field() { sed -n "s/^$2: //p" "$1" | head -n1; }

send() {
  local from="" to="" issue="none" type="" subject=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --from) from="$2" ;; --to) to="$2" ;; --issue) issue="$2" ;;
      --type) type="$2" ;; --subject) subject="$2" ;;
      *) fail "unknown option '$1'" ;;
    esac
    shift 2
  done
  valid_agent "$from"; valid_agent "$to"
  [[ "$type" =~ $type_pattern ]] || fail "type must match $type_pattern"
  [[ -n "$subject" ]] || fail "subject is required"
  local id dir
  id="$(date -u +%Y%m%dT%H%M%S%N | cut -c1-21)-$from"
  dir="$bus/inbox/$to"
  mkdir -p "$dir"
  {
    echo "id: $id"; echo "from: $from"; echo "to: $to"; echo "issue: $issue"
    echo "type: $type"; echo "subject: $subject"; echo "---"
    cat
  } > "$dir/$id.md"
  echo "$id"
}

inbox() {
  valid_agent "$1"
  local file
  for file in "$bus/inbox/$1"/*.md; do
    [[ -e "$file" ]] || { echo "(no unread messages for $1)"; return; }
    printf '%s | from %s | #%s | %s | %s\n' "$(field "$file" id)" "$(field "$file" from)" \
      "$(field "$file" issue)" "$(field "$file" type)" "$(field "$file" subject)"
  done
}

read_message() {
  valid_agent "$1"
  local file="$bus/inbox/$1/$2.md"
  [[ -e "$file" ]] || file="$bus/archive/$1/$2.md"
  [[ -e "$file" ]] || fail "no message $2 for $1"
  cat "$file"
}

ack() {
  valid_agent "$1"
  local file="$bus/inbox/$1/$2.md"
  [[ -e "$file" ]] || fail "no unread message $2 for $1"
  mkdir -p "$bus/archive/$1"
  mv "$file" "$bus/archive/$1/"
}

log() {
  local issue="${1:-}" file
  for file in "$bus"/inbox/*/*.md "$bus"/archive/*/*.md; do
    [[ -e "$file" ]] || continue
    [[ -z "$issue" || "$(field "$file" issue)" == "$issue" ]] || continue
    printf '%s | %s -> %s | #%s | %s | %s\n' "$(field "$file" id)" "$(field "$file" from)" \
      "$(field "$file" to)" "$(field "$file" issue)" "$(field "$file" type)" "$(field "$file" subject)"
  done | sort
}

usage() {
  cat <<USAGE
usage:
  bus.sh send --from A --to B --issue N --type T --subject "text"   (body on stdin)
  bus.sh inbox AGENT            list unread messages
  bus.sh read AGENT ID          print a message (unread or archived)
  bus.sh ack AGENT ID           mark a message as handled
  bus.sh log [ISSUE]            all messages, oldest first
types: handoff question answer feedback bug approval-request approved rejected report
USAGE
}

case "${1:-}" in
  send) shift; send "$@" ;;
  inbox) shift; inbox "${1:?agent}" ;;
  read) shift; read_message "${1:?agent}" "${2:?id}" ;;
  ack) shift; ack "${1:?agent}" "${2:?id}" ;;
  log) shift; log "${1:-}" ;;
  *) usage; exit 1 ;;
esac
