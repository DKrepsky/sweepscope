<script module lang="ts">
  import type { Snippet } from 'svelte';

  export interface RunButtonProps {
    running: boolean;
    disabled?: boolean;
    disabledReason?: string;
    onclick: () => void;
    /** Replaces the default `Run` / `Stop` label. */
    children?: Snippet;
  }
</script>

<script lang="ts">
  import Button from '../button/Button.svelte';
  import Icon from '../icon/Icon.svelte';

  let { running, disabled, disabledReason, onclick, children }: RunButtonProps = $props();
</script>

<Button size="lg" tone={running ? 'stop' : 'run'} {disabled} {disabledReason} {onclick}>
  {#snippet left()}
    <Icon name={running ? 'stop' : 'play'} />
  {/snippet}
  {#if children}{@render children()}{:else}{running ? 'Stop' : 'Run'}{/if}
</Button>
