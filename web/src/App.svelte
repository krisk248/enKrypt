<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import StatusBar from './lib/components/StatusBar.svelte';
  import Toasts from './lib/components/Toasts.svelte';
  import Spinner from './lib/components/Spinner.svelte';
  import FirstRunDialog from './lib/views/FirstRunDialog.svelte';
  import VaultUnlock from './lib/views/VaultUnlock.svelte';
  import KeysView from './lib/views/KeysView.svelte';
  import EncryptView from './lib/views/EncryptView.svelte';
  import DecryptView from './lib/views/DecryptView.svelte';
  import SignVerifyView from './lib/views/SignVerifyView.svelte';
  import TextView from './lib/views/TextView.svelte';
  import SettingsView from './lib/views/SettingsView.svelte';
  import { keyring } from './lib/stores/keyring.svelte';
  import { nav } from './lib/stores/nav.svelte';

  onMount(() => {
    keyring.init();
  });
</script>

<Toasts />

{#if !keyring.ready}
  <div class="flex h-full items-center justify-center text-slate-400">
    <Spinner size={28} />
  </div>
{:else if keyring.needsFirstRun}
  <FirstRunDialog />
{:else if keyring.locked}
  <VaultUnlock />
{:else}
  <div class="flex h-full flex-col">
    <div class="flex min-h-0 flex-1">
      <Sidebar current={nav.view} onNavigate={(v) => nav.go(v as never)} />
      <main class="min-w-0 flex-1 overflow-y-auto">
        {#if nav.view === 'keys'}
          <KeysView />
        {:else if nav.view === 'encrypt'}
          <EncryptView />
        {:else if nav.view === 'decrypt'}
          <DecryptView />
        {:else if nav.view === 'sign'}
          <SignVerifyView />
        {:else if nav.view === 'text'}
          <TextView />
        {:else if nav.view === 'settings'}
          <SettingsView />
        {/if}
      </main>
    </div>
    <StatusBar />
  </div>
{/if}
