<script module lang="ts">
  import type { IconName } from '../../design-system';

  export interface IconProps {
    name: IconName;
  }
</script>

<script lang="ts">
  import { icons } from '../../design-system';

  let { name }: IconProps = $props();

  const definition = $derived(icons[name]);
</script>

<svg class="icon" viewBox={definition.viewBox} aria-hidden="true" focusable="false">
  {#each definition.shapes as shape, index (index)}
    <path d={shape.d} class:filled={shape.filled} stroke-width={shape.strokeWidth} stroke-dasharray={shape.dashArray} />
  {/each}
</svg>

<style>
  .icon {
    flex: none;
    width: var(--ss-size-icon);
    height: var(--ss-size-icon);
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: square;
    stroke-linejoin: miter;
  }

  .filled {
    fill: currentColor;
    stroke: none;
  }
</style>
