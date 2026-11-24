<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { CalibrationTagModel } from '../../app/calibration';
  import Icon from '../../ui/icon/Icon.svelte';

  interface Props {
    tag: CalibrationTagModel;
    onopen: () => void;
    /** Hover delay in milliseconds; keyboard focus shows the tooltip at once. */
    delay?: number;
  }

  const { tag, onopen, delay = 500 }: Props = $props();

  const id = $props.id();
  let visible = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const label = $derived(
    tag.state === 'calibrated'
      ? 'Calibrated. Open calibration dialog'
      : tag.state === 'none'
        ? 'No calibration. Open calibration dialog'
        : 'Calibration unavailable',
  );
  const text = $derived(
    tag.state === 'calibrated' ? 'Calibrated' : tag.state === 'none' ? 'No calibration' : 'Calibration -',
  );
  const heading = $derived(
    tag.state === 'calibrated' ? (tag.storedAt ? `Calibrated ${tag.storedAt}` : 'Calibrated') : 'No calibration',
  );
  const lead = $derived(tag.state === 'unavailable' ? undefined : tag.tooltip.heading);
  const lines = $derived(tag.state === 'unavailable' ? [] : tag.tooltip.lines.map(splitLine));
  const hint = $derived(tag.state === 'calibrated' ? 'Click to calibrate again.' : 'Click to calibrate.');

  /** A line is `CHn text`; the name takes the channel colour. */
  function splitLine(line: string) {
    const [name = '', ...rest] = line.split(' ');
    return { name, text: rest.join(' '), color: `var(--ss-color-ch${name.slice(2)})` };
  }

  function cancelPending() {
    clearTimeout(timer);
    timer = undefined;
  }

  function show() {
    cancelPending();
    visible = true;
  }

  function hide() {
    cancelPending();
    visible = false;
  }

  function showAfterDelay() {
    cancelPending();
    timer = setTimeout(show, delay);
  }

  function hideOnEscape(event: KeyboardEvent) {
    if (visible && event.key === 'Escape') hide();
  }

  onDestroy(cancelPending);
</script>

<svelte:window onkeydown={hideOnEscape} />

{#if tag.state === 'unavailable'}
  <button class="caltag na" type="button" disabled aria-disabled="true" title={tag.reason}>{text}</button>
{:else}
  <span
    class="host"
    role="presentation"
    onmouseenter={showAfterDelay}
    onmouseleave={hide}
    onfocusin={show}
    onfocusout={hide}
  >
    <button
      class={['caltag', tag.state === 'calibrated' ? 'ok' : 'none']}
      type="button"
      aria-label={label}
      aria-describedby={visible ? id : undefined}
      onclick={onopen}
    >
      {#if tag.state === 'calibrated'}
        <svg class="check" viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8.5l3.2 3L13 4.5" /></svg>
      {:else}
        <Icon name="warning" />
      {/if}
      {text}
    </button>
    {#if visible}
      <div class="tip" role="tooltip" {id}>
        <b>{heading}</b>
        {#if lead}<span>{lead}</span>{/if}
        {#if lines.length > 0}
          <ul>
            {#each lines as line (line.name)}
              <li style:--chc={line.color}><b class="ch">{line.name}</b> <span class="ss-num">{line.text}</span></li>
            {/each}
          </ul>
        {:else}
          <span class="empty">No channel enabled</span>
        {/if}
        <span>{hint}</span>
      </div>
    {/if}
  </span>
{/if}

<style>
  .host {
    position: relative;
    display: inline-flex;
  }
  .caltag {
    display: inline-flex;
    align-items: center;
    gap: var(--ss-space-3);
    block-size: var(--ss-size-control-sm);
    padding: 0 var(--ss-space-4);
    border: var(--ss-border-width) solid currentColor;
    border-radius: var(--ss-radius-sm);
    background: var(--ss-color-bg-inset);
    font: var(--ss-font-weight-bold) var(--ss-font-size-xs) / 1 var(--ss-font-ui);
    letter-spacing: var(--ss-letter-spacing-caps);
    text-transform: uppercase;
    white-space: nowrap;
    cursor: pointer;
  }
  .caltag:hover:not(:disabled) {
    background: var(--ss-color-bg-hover);
  }
  .caltag:focus-visible {
    outline: var(--ss-border-width-focus) solid var(--ss-color-focus);
    outline-offset: var(--ss-border-width);
  }
  .ok {
    color: var(--ss-color-status-ok);
  }
  .none {
    color: var(--ss-color-status-warning);
  }
  .na {
    color: var(--ss-color-text-disabled);
    cursor: not-allowed;
    border-color: var(--ss-color-border-subtle);
  }
  .check {
    inline-size: 1em;
    block-size: 1em;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .tip {
    position: absolute;
    inset-block-end: 100%;
    inset-inline-end: 0;
    z-index: var(--ss-z-popover);
    margin-block-end: var(--ss-space-2);
    inline-size: 21rem;
    max-inline-size: 90vw;
    display: grid;
    gap: var(--ss-space-2);
    padding: var(--ss-space-3) var(--ss-space-4);
    background: var(--ss-color-bg-raised);
    border: var(--ss-border-width) solid var(--ss-color-border-strong);
    border-radius: var(--ss-radius-sm);
    box-shadow: var(--ss-shadow-popover);
    color: var(--ss-color-text-secondary);
    font-size: var(--ss-font-size-xs);
    font-weight: var(--ss-font-weight-regular);
    letter-spacing: normal;
    text-transform: none;
    white-space: normal;
    pointer-events: none;
  }
  .tip b {
    color: var(--ss-color-text-primary);
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: var(--ss-space-1);
  }
  .tip .ch {
    color: var(--chc);
    font-family: var(--ss-font-mono);
  }
  li span {
    color: var(--ss-color-text-primary);
  }
  .empty {
    color: var(--ss-color-text-muted);
  }
</style>
