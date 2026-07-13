<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';

  interface Props {
    open: boolean;
    title: string;
    onClose: () => void;
    size?: 'sm' | 'md' | 'lg';
    children: Snippet;
    footer?: Snippet;
  }
  let { open, title, onClose, size = 'md', children, footer }: Props = $props();

  let dialogEl = $state<HTMLDivElement | null>(null);

  const widths = { sm: 'max-w-sm', md: 'max-w-lg', lg: 'max-w-2xl' };

  function trapFocus(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
      return;
    }
    if (e.key !== 'Tab' || !dialogEl) return;
    const focusables = dialogEl.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), textarea, input, select, [tabindex]:not([tabindex="-1"])',
    );
    if (focusables.length === 0) return;
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  }

  $effect(() => {
    if (open && dialogEl) {
      // Focus the first focusable element when opened.
      const el = dialogEl.querySelector<HTMLElement>(
        'input, textarea, button, select, a[href]',
      );
      el?.focus();
    }
  });
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    role="presentation"
    onkeydown={trapFocus}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-slate-900/50 backdrop-blur-sm"
      aria-label="Close dialog"
      onclick={onClose}
      tabindex="-1"
    ></button>

    <div
      bind:this={dialogEl}
      class="card relative z-10 w-full {widths[size]} animate-fade-in overflow-hidden"
      role="dialog"
      aria-modal="true"
      aria-label={title}
    >
      <header class="flex items-center justify-between border-b border-slate-200 px-5 py-3.5 dark:border-slate-800">
        <h2 class="text-base font-semibold">{title}</h2>
        <button class="btn-ghost -mr-2 p-1.5" onclick={onClose} aria-label="Close">
          <Icon name="x" size={18} />
        </button>
      </header>
      <div class="max-h-[70vh] overflow-y-auto px-5 py-4">
        {@render children()}
      </div>
      {#if footer}
        <footer class="flex justify-end gap-2 border-t border-slate-200 px-5 py-3 dark:border-slate-800">
          {@render footer()}
        </footer>
      {/if}
    </div>
  </div>
{/if}
