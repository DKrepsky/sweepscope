<script lang="ts">
  import { Banner, BannerStack } from '../../ui/banner';
  import { Button } from '../../ui/button';
  import { Dialog } from '../../ui/dialog';
  import { Menu } from '../../ui/menu';
  import { Panel } from '../../ui/panel';
  import { Toast } from '../../ui/toast';
  import { menuItems } from './mock-data';
  import Row from './Row.svelte';
  import Section from './Section.svelte';

  const noop = () => {};
  let collapsed = $state(true);
  let dialog = $state<'none' | 'modal' | 'task'>('none');
  let menuAnchor = $state<{ x: number; y: number } | null>(null);
  let toastShown = $state(true);
  let selected = $state('none');

  let menuHost: HTMLElement | undefined = $state();

  function openMenu() {
    const box = menuHost?.getBoundingClientRect() ?? { left: 0, bottom: 0 };
    menuAnchor = { x: box.left, y: box.bottom };
  }
</script>

<Section component="Panel" title="Panel and task panel" note="Normal and collapsed.">
  <div class="stack">
    <Panel>
      {#snippet header()}Measurements{/snippet}
      {#snippet body()}Panel body (illustrative){/snippet}
      {#snippet actions()}<Button size="sm">Action</Button>{/snippet}
    </Panel>
    <Panel {collapsed} ontoggle={() => (collapsed = !collapsed)}>
      {#snippet header()}Collapsed task panel{/snippet}
      {#snippet body()}Hidden until expanded{/snippet}
    </Panel>
  </div>
</Section>

<Section component="Banner" title="Banner and banner stack">
  <div class="stack">
    <Banner kind="error">
      {#snippet text()}<b>No permission to open the scope.</b> Install the udev rule, then replug (illustrative).{/snippet}
      {#snippet actions()}<Button>Copy command</Button><Button>Retry</Button>{/snippet}
    </Banner>
    <Banner kind="warning" ondismiss={noop}>
      {#snippet text()}Aliasing risk: message text from the backend (illustrative).{/snippet}
    </Banner>
    <Banner kind="info" ondismiss={noop}>
      {#snippet text()}Informational banner (illustrative).{/snippet}
    </Banner>
    <BannerStack
      banners={[
        { id: 'a', node: oldest },
        { id: 'b', node: middle },
        { id: 'c', node: newest },
      ]}
    />
  </div>
</Section>

{#snippet oldest()}<Banner kind="info">{#snippet text()}Oldest, hidden (max two){/snippet}</Banner>{/snippet}
{#snippet middle()}<Banner kind="warning">{#snippet text()}Stack: older of two visible{/snippet}</Banner>{/snippet}
{#snippet newest()}<Banner kind="error">{#snippet text()}Stack: newest, shown first{/snippet}</Banner>{/snippet}

<Section component="Toast" note="Disappears on its own after 4 s and never takes focus.">
  <Row>
    {#if toastShown}
      <Toast kind="info" ondismiss={() => (toastShown = false)}>
        {#snippet text()}Setup saved (illustrative){/snippet}
      </Toast>
    {/if}
    <Button onclick={() => (toastShown = true)}>Show toast</Button>
  </Row>
</Section>

<Section component="Dialog" note="Modal only while acquisition is stopped; the task panel variant does not block.">
  <Row>
    <Button onclick={() => (dialog = 'modal')}>Open modal dialog</Button>
    <Button onclick={() => (dialog = 'task')}>Open non-modal dialog</Button>
  </Row>
  <Dialog open={dialog === 'modal'} modal onclose={() => (dialog = 'none')}>
    {#snippet title()}Zero calibration &middot; CH2 (illustrative){/snippet}
    {#snippet body()}Short the input, then press Measure.{/snippet}
    {#snippet actions()}<Button onclick={() => (dialog = 'none')}>Close</Button>{/snippet}
  </Dialog>
  <Dialog open={dialog === 'task'} modal={false} onclose={() => (dialog = 'none')}>
    {#snippet title()}Task panel (illustrative){/snippet}
    {#snippet body()}Acquisition keeps running.{/snippet}
    {#snippet actions()}<Button onclick={() => (dialog = 'none')}>Close</Button>{/snippet}
  </Dialog>
</Section>

<Section component="Menu" note="Items: normal, with badge, disabled with a reason.">
  <Row>
    <span bind:this={menuHost}><Button onclick={openMenu}>Open menu</Button></span>
    <span>Selected: {selected}</span>
  </Row>
  {#if menuAnchor}
    <Menu
      items={menuItems}
      open
      anchor={menuAnchor}
      onclose={() => (menuAnchor = null)}
      onselect={(id) => {
        selected = id;
        menuAnchor = null;
      }}
    />
  {/if}
</Section>

<style>
  .stack {
    display: grid;
    gap: var(--ss-space-4);
  }
</style>
