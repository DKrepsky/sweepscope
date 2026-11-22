<script lang="ts">
  import type { Availability } from '../../app/main-window';
  import Stepper from '../../ui/stepper/Stepper.svelte';
  import Tooltip from '../../ui/tooltip/Tooltip.svelte';

  interface Props {
    value: string;
    unit?: string;
    valueText: string;
    ariaLabel: string;
    /** Larger value (+1) and smaller value (-1); a blocked direction explains itself in a tooltip. */
    larger?: Availability;
    smaller?: Availability;
    /** The whole control is blocked, for example no scope; `reason` wins over the per-direction ones. */
    off?: boolean;
    reason?: string;
    onstep: (direction: -1 | 1) => void;
    onreset?: () => void;
  }

  const { value, unit, valueText, ariaLabel, larger, smaller, off = false, reason, onstep, onreset }: Props = $props();

  const blockedReasons = $derived(
    [smaller, larger].flatMap((side) => (side && !side.enabled && side.reason ? [side.reason] : [])),
  );
  const tip = $derived(off ? reason : blockedReasons.length > 0 ? [...new Set(blockedReasons)].join(' · ') : undefined);
</script>

{#snippet stepper()}
  <Stepper
    {value}
    {unit}
    {valueText}
    {ariaLabel}
    {off}
    atMin={smaller?.enabled === false}
    atMax={larger?.enabled === false}
    onstep={(direction) => onstep(direction)}
    {onreset}
  />
{/snippet}

{#if tip}
  <Tooltip text={tip}>{@render stepper()}</Tooltip>
{:else}
  {@render stepper()}
{/if}
