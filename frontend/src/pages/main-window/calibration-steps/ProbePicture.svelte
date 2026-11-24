<script lang="ts">
  // Static picture; the script block lets svelte-check resolve the import.
</script>

<svg class="picture" viewBox="0 0 300 64" role="img" aria-label="Probe tip touching the probe ground clip">
  <g class="wires">
    <path d="M20 24H110l10 6h30" />
    <path d="M20 44H100c10 0 14-8 24-8h26" />
    <rect x="150" y="26" width="26" height="16" class="plug" />
    <path d="M176 30h18M176 38h18" />
    <path d="M194 30v8" class="join" />
  </g>
  <text x="206" y="26" font-size="10">tip</text>
  <text x="206" y="48" font-size="10">ground clip</text>
  <text x="20" y="14" font-size="10">Tip and ground clip joined</text>
</svg>

<style>
  .picture {
    inline-size: 100%;
    block-size: 4rem;
    background: var(--ss-color-bg-inset);
    border: var(--ss-border-width) solid var(--ss-color-border-subtle);
  }
  .wires {
    fill: none;
    stroke: var(--ss-color-text-secondary);
    stroke-width: 2;
  }
  .plug {
    fill: var(--ss-color-bg-raised);
  }
  .join {
    stroke: var(--ss-color-status-ok);
    stroke-width: 3;
  }
  text {
    fill: var(--ss-color-text-muted);
  }
</style>
