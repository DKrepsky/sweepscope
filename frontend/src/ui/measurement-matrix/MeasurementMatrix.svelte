<script lang="ts">
  import type { Snippet } from 'svelte';
  import Tooltip from '../tooltip/Tooltip.svelte';
  import type { MeasurementCell, MeasurementColumn, MeasurementFootnote, MeasurementRow } from './types';

  interface Props {
    columns: MeasurementColumn[];
    rows: MeasurementRow[];
    frozenLabel?: 'Frozen' | 'Offline';
    footnotes: MeasurementFootnote[];
    cell?: Snippet<[MeasurementCell]>;
    /** Rendered after the label inside the column header, for example an alias badge. */
    columnHeader?: Snippet<[MeasurementColumn]>;
  }

  const { columns, rows, frozenLabel, footnotes, cell, columnHeader }: Props = $props();

  const isMuted = (c: MeasurementCell) => frozenLabel !== undefined || c.state === 'invalid' || c.state === 'stale';
  const shownText = (c: MeasurementCell) => (c.state === 'invalid' ? '-' : c.text);
</script>

{#snippet content(c: MeasurementCell)}
  {#if cell}
    {@render cell(c)}
  {:else}
    <span class="ss-num">{shownText(c)}</span>
    {#if c.state !== 'invalid' && c.unit}<span class="unit">{c.unit}</span>{/if}
    {#if c.footnote !== undefined}<sup>[{c.footnote}]</sup>{/if}
  {/if}
{/snippet}

<table>
  <thead>
    <tr>
      <th scope="col" class="corner">{frozenLabel ?? ''}</th>
      {#each columns as column (column.id)}
        <th scope="col" class="ss-num channel" style:--column-colour="var(--ss-{column.color})">
          {column.label}{@render columnHeader?.(column)}
        </th>
      {/each}
    </tr>
  </thead>
  <tbody>
    {#each rows as row (row.id)}
      <tr>
        <th scope="row">{row.label}</th>
        {#each columns as column (column.id)}
          {@const c = row.cells[column.id]}
          <td class={c ? [c.state, c.warning && 'warning', isMuted(c) && 'muted'] : 'empty'}>
            {#if c?.tooltip}
              <Tooltip text={c.tooltip}>
                <!-- Keyboard users need focus to read the tooltip. -->
                <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
                <span tabindex="0">{@render content(c)}</span>
              </Tooltip>
            {:else if c}
              {@render content(c)}
            {/if}
          </td>
        {/each}
      </tr>
    {/each}
  </tbody>
</table>

{#if footnotes.length > 0}
  <ol class="footnotes">
    {#each footnotes as note (note.key)}
      <li><sup>[{note.key}]</sup> {note.text}</li>
    {/each}
  </ol>
{/if}

<style>
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: var(--ss-font-size-md);
  }
  th,
  td {
    padding: var(--ss-space-1) var(--ss-space-3);
    border-bottom: var(--ss-border-width) solid var(--ss-color-border-subtle);
    text-align: right;
    height: var(--ss-size-control-sm);
  }
  th[scope='row'],
  .corner {
    text-align: left;
    font-weight: var(--ss-font-weight-regular);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
  .corner {
    text-transform: uppercase;
    letter-spacing: var(--ss-letter-spacing-caps);
    color: var(--ss-color-text-muted);
  }
  .channel {
    text-align: center;
    color: var(--ss-color-text-on-accent);
    background: var(--column-colour);
    border-bottom: 0;
  }
  td.muted {
    color: var(--ss-color-text-muted);
  }
  td.warning {
    box-shadow: inset var(--ss-space-1) 0 0 var(--ss-color-status-warning);
  }
  sup {
    font-family: var(--ss-font-mono);
    color: var(--ss-color-status-warning);
  }
  sup {
    font-size: var(--ss-font-size-2xs);
    margin-left: var(--ss-space-1);
  }
  .unit {
    color: var(--ss-color-text-muted);
    font-size: var(--ss-font-size-xs);
    margin-left: var(--ss-space-1);
  }
  .footnotes {
    list-style: none;
    padding: 0;
    margin-top: var(--ss-space-3);
    display: grid;
    gap: var(--ss-space-2);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-secondary);
  }
</style>
