<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    icon?: string;
    title: string;
    description?: string;
    children?: Snippet;
  }
  let { icon = 'info', title, description = '', children }: Props = $props();
</script>

<div class="flex flex-col items-center justify-center gap-3 px-6 py-16 text-center">
  <div class="rounded-2xl bg-slate-100 p-4 text-slate-400 dark:bg-slate-800/70 dark:text-slate-500">
    <Icon name={icon} size={32} />
  </div>
  <h3 class="text-base font-semibold text-slate-700 dark:text-slate-200">{title}</h3>
  {#if description}
    <p class="max-w-sm text-sm text-slate-500 dark:text-slate-400">{description}</p>
  {/if}
  {#if children}
    <div class="mt-2">{@render children()}</div>
  {/if}
</div>
