<script lang="ts">
  import type { CalibrationDialogModel } from '../../../app/calibration';
  import Note from './Note.svelte';
  import OffsetTable from './OffsetTable.svelte';

  const { table }: { table: CalibrationDialogModel['table'] } = $props();
</script>

<Note kind="good" role="status" focusTarget>
  <b>Calibration completed</b>
  <br />Offsets applied per channel and range and kept after restart, the device is now CALIBRATED
</Note>
{#if table}<OffsetTable rows={table} />{/if}
