<script lang="ts">
  import { tokenNames, tokens } from '../../design-system';
  import type { TokenName } from '../../design-system';
  import Row from './Row.svelte';
  import Section from './Section.svelte';

  const colourTokens = tokenNames.filter((name) => name.startsWith('color-'));
  const textTokens: TokenName[] = [
    'color-text-primary',
    'color-text-secondary',
    'color-text-muted',
    'color-text-disabled',
    'color-accent',
  ];
</script>

<Section component="Tokens" title="Tokens: colours (generated from tokens.css)">
  <Row>
    {#each colourTokens as name (name)}
      <div class="swatch-item" data-swatch={name}>
        <span class="swatch" style="background: var(--ss-{name})"></span>
        <span>{name.replace('color-', '')}</span>
        <span class="value">{tokens[name]}</span>
      </div>
    {/each}
  </Row>
  <Row label="Text">
    {#each textTokens as name (name)}
      <span style="color: var(--ss-{name})">{name.replace('color-', '')}</span>
    {/each}
  </Row>
</Section>

<style>
  .swatch-item {
    display: grid;
    gap: var(--ss-space-2);
    inline-size: 8rem;
    font-size: var(--ss-font-size-xs);
  }

  .swatch {
    display: block;
    block-size: var(--ss-size-control-md);
    border: var(--ss-border-width) solid var(--ss-color-border);
  }

  .value {
    color: var(--ss-color-text-muted);
    font-family: var(--ss-font-mono);
    font-size: var(--ss-font-size-2xs);
  }
</style>
