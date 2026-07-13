<script lang="ts">
  import Icon from './Icon.svelte';
  import { theme } from '../stores/theme.svelte';
  import { keyring } from '../stores/keyring.svelte';

  interface Props {
    current: string;
    onNavigate: (view: string) => void;
  }
  let { current, onNavigate }: Props = $props();

  const nav = [
    { id: 'keys', label: 'Keys', icon: 'keys' },
    { id: 'encrypt', label: 'Encrypt', icon: 'lock' },
    { id: 'decrypt', label: 'Decrypt', icon: 'unlock' },
    { id: 'sign', label: 'Sign / Verify', icon: 'sign' },
    { id: 'text', label: 'Text', icon: 'text' },
    { id: 'settings', label: 'Settings', icon: 'settings' },
  ];
</script>

<aside class="flex w-60 shrink-0 flex-col border-r border-slate-200 bg-white/60 dark:border-slate-800 dark:bg-slate-900/40">
  <div class="flex items-center gap-2.5 px-5 py-4">
    <div class="grid h-9 w-9 place-items-center rounded-lg bg-gradient-to-b from-brand-400 to-brand-600 text-white shadow-sm">
      <Icon name="shield" size={20} fill />
    </div>
    <div>
      <h1 class="text-sm font-bold leading-none">enKrypt</h1>
      <p class="mt-0.5 text-[0.7rem] text-slate-500 dark:text-slate-400">Privacy Tray</p>
    </div>
  </div>

  <nav class="flex-1 space-y-1 px-3 py-2">
    {#each nav as item}
      <button
        class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors
          {current === item.id
          ? 'bg-brand-50 text-brand-700 dark:bg-brand-950/60 dark:text-brand-200'
          : 'text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800/70'}"
        onclick={() => onNavigate(item.id)}
        aria-current={current === item.id ? 'page' : undefined}
      >
        <Icon name={item.icon} size={19} />
        {item.label}
        {#if item.id === 'keys' && keyring.keys.length > 0}
          <span class="ml-auto rounded-full bg-slate-200 px-1.5 text-xs text-slate-600 dark:bg-slate-700 dark:text-slate-300">
            {keyring.keys.length}
          </span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="border-t border-slate-200 p-3 dark:border-slate-800">
    {#if keyring.storageMode === 'session'}
      <div class="mb-2 flex items-center gap-2 rounded-lg bg-amber-50 px-2.5 py-1.5 text-xs text-amber-700 dark:bg-amber-950/50 dark:text-amber-300">
        <Icon name="warn" size={15} class="shrink-0" />
        <span>Session-only — export before closing.</span>
      </div>
    {/if}
    <button class="btn-ghost w-full justify-start" onclick={() => theme.toggle()}>
      <Icon name={theme.isDark ? 'sun' : 'moon'} size={18} />
      {theme.isDark ? 'Light mode' : 'Dark mode'}
    </button>
  </div>
</aside>
