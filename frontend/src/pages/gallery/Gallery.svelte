<script lang="ts">
  import ControlSections from './ControlSections.svelte';
  import FeedbackSections from './FeedbackSections.svelte';
  import IndicatorSections from './IndicatorSections.svelte';
  import { ILLUSTRATIVE } from './mock-data';
  import ScopeSections from './ScopeSections.svelte';
  import TokensSection from './TokensSection.svelte';

  /** Only `dark` exists; the attribute is the hook for future themes. */
  const { theme = 'dark' }: { theme?: string } = $props();

  $effect(() => {
    document.documentElement.dataset.theme = theme;
  });
</script>

<main class="gallery" data-testid="gallery">
  <h1>SweepScope component gallery</h1>
  <p class="note">{ILLUSTRATIVE}</p>
  <TokensSection />
  <ControlSections />
  <IndicatorSections />
  <ScopeSections />
  <FeedbackSections />
</main>

<style>
  .gallery {
    display: grid;
    gap: var(--ss-space-6);
    max-inline-size: 76rem;
    block-size: 100vh;
    padding: var(--ss-space-6);
    overflow: auto;
    align-content: start;
  }

  h1 {
    font-size: var(--ss-font-size-xl);
  }

  .note {
    color: var(--ss-color-text-muted);
  }
</style>
