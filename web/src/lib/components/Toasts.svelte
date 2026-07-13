<script lang="ts">
  import { toasts } from '../stores/toast.svelte';
  import Icon from './Icon.svelte';

  const kindStyles: Record<string, string> = {
    success: 'border-green-500/30 bg-green-50 text-green-800 dark:bg-green-950/60 dark:text-green-200',
    error: 'border-red-500/30 bg-red-50 text-red-800 dark:bg-red-950/60 dark:text-red-200',
    info: 'border-brand-500/30 bg-brand-50 text-brand-800 dark:bg-brand-950/60 dark:text-brand-200',
  };
  const kindIcon: Record<string, string> = {
    success: 'check',
    error: 'warn',
    info: 'info',
  };
</script>

<div class="pointer-events-none fixed bottom-4 right-4 z-[60] flex w-80 flex-col gap-2">
  {#each toasts.items as toast (toast.id)}
    <div
      class="pointer-events-auto flex items-start gap-2.5 rounded-lg border px-3.5 py-2.5 text-sm shadow-lg animate-fade-in {kindStyles[toast.kind]}"
      role="status"
    >
      <Icon name={kindIcon[toast.kind]} size={18} class="mt-0.5 shrink-0" />
      <span class="flex-1 leading-snug">{toast.message}</span>
      <button class="shrink-0 opacity-60 hover:opacity-100" onclick={() => toasts.dismiss(toast.id)} aria-label="Dismiss">
        <Icon name="x" size={16} />
      </button>
    </div>
  {/each}
</div>
