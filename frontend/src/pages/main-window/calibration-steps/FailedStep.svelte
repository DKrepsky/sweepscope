<script lang="ts">
  import Note from './Note.svelte';

  const { failure }: { failure: string } = $props();
</script>

<Note kind="error" role="alert" focusTarget>{failure}</Note>
