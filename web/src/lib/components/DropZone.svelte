<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    onFiles: (files: File[]) => void;
    accept?: string;
    multiple?: boolean;
    hint?: string;
    icon?: string;
    children?: Snippet;
  }
  let {
    onFiles,
    accept = '',
    multiple = false,
    hint = 'Drop a file here, or click to browse',
    icon = 'file',
    children,
  }: Props = $props();

  let dragging = $state(false);
  let inputEl = $state<HTMLInputElement | null>(null);

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragging = false;
    const files = Array.from(e.dataTransfer?.files ?? []);
    if (files.length) onFiles(multiple ? files : [files[0]]);
  }

  function handleSelect(e: Event) {
    const files = Array.from((e.target as HTMLInputElement).files ?? []);
    if (files.length) onFiles(multiple ? files : [files[0]]);
    (e.target as HTMLInputElement).value = '';
  }
</script>

<button
  type="button"
  class="flex w-full flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed px-6 py-10 text-center transition-colors
    {dragging
    ? 'border-brand-500 bg-brand-50 dark:bg-brand-950/40'
    : 'border-slate-300 hover:border-brand-400 hover:bg-slate-50 dark:border-slate-700 dark:hover:bg-slate-900'}"
  ondragover={(e) => {
    e.preventDefault();
    dragging = true;
  }}
  ondragleave={() => (dragging = false)}
  ondrop={handleDrop}
  onclick={() => inputEl?.click()}
>
  <div class="rounded-full bg-brand-100 p-3 text-brand-600 dark:bg-brand-900/50 dark:text-brand-300">
    <Icon name={icon} size={24} />
  </div>
  {#if children}
    {@render children()}
  {:else}
    <p class="text-sm text-slate-600 dark:text-slate-300">{hint}</p>
  {/if}
  <input bind:this={inputEl} type="file" {accept} {multiple} class="hidden" onchange={handleSelect} />
</button>
