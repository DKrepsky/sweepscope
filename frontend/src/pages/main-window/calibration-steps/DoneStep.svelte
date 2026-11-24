<script lang="ts">
  import type { CalibrationDialogModel } from '../../../app/calibration';
  import { channelNumbers } from './channels';
  import Note from './Note.svelte';
  import OffsetTable from './OffsetTable.svelte';

  const {
    table,
    overridden,
  }: { table: CalibrationDialogModel['table']; overridden: CalibrationDialogModel['overridden'] } = $props();

  const accepted = $derived(channelNumbers.filter((number) => overridden[number - 1]));
</script>

<Note kind="good" role="status" focusTarget>
  <b>Calibration completed</b>
  <br />Offsets applied per channel and range and kept after restart, the device is now CALIBRATED
</Note>
{#if table}<OffsetTable rows={table} />{/if}
{#if accepted.length > 0}
  <ul class="accepted">
    {#each accepted as number (number)}
      <li>CH{number} accepted although the input did not look shorted</li>
    {/each}
  </ul>
{/if}

<style>
  .accepted {
    margin: 0;
    padding: 0;
    list-style: none;
    display: grid;
    gap: var(--ss-space-2);
    font-size: var(--ss-font-size-xs);
    color: var(--ss-color-text-muted);
  }
</style>
