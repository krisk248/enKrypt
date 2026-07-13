<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import Dialog from '../components/Dialog.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import { theme } from '../stores/theme.svelte';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { cryptoApi } from '../worker/rpc';
  import { onMount } from 'svelte';

  let coreVersion = $state('');
  let showWipe = $state(false);
  let showVault = $state(false);
  let vaultPass = $state('');
  let vaultPass2 = $state('');

  onMount(async () => {
    try { coreVersion = await cryptoApi.version(); } catch { /* ignore */ }
  });

  async function setMode(mode: 'session' | 'persistent') {
    await keyring.setStorageMode(mode);
    toasts.success(`Storage set to ${mode}.`);
  }

  async function enableVault() {
    if (vaultPass.length < 8) return toasts.error('Vault passphrase must be at least 8 characters.');
    if (vaultPass !== vaultPass2) return toasts.error('Passphrases do not match.');
    await keyring.enableVault(vaultPass);
    vaultPass = vaultPass2 = '';
    showVault = false;
    toasts.success('Vault enabled. Your keyring is now sealed at rest.');
  }

  async function disableVault() {
    await keyring.disableVault();
    toasts.info('Vault disabled. Keys are stored as OpenPGP-encrypted packets.');
  }

  async function wipe() {
    await keyring.wipe();
    showWipe = false;
    toasts.success('All local data wiped.');
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 p-6">
  <h2 class="text-lg font-bold">Settings</h2>

  <section class="card p-5">
    <h3 class="mb-3 flex items-center gap-2 font-semibold"><Icon name="sun" size={18} /> Appearance</h3>
    <div class="flex gap-2">
      {#each ['light', 'dark', 'system'] as m}
        <button class="btn-secondary flex-1 capitalize {theme.mode === m ? '!border-brand-500 !text-brand-600' : ''}" onclick={() => theme.set(m as never)}>{m}</button>
      {/each}
    </div>
  </section>

  <section class="card p-5">
    <h3 class="mb-3 flex items-center gap-2 font-semibold"><Icon name="lock" size={18} /> Key storage</h3>
    <div class="space-y-3">
      <label class="flex cursor-pointer items-start gap-3 rounded-lg border border-slate-200 p-3 dark:border-slate-700">
        <input type="radio" name="mode" class="mt-1 accent-brand-600" checked={keyring.storageMode === 'session'} onchange={() => setMode('session')} />
        <div><p class="font-medium">Session only</p><p class="text-xs text-slate-500 dark:text-slate-400">Keys held in memory; nothing written to disk.</p></div>
      </label>
      <label class="flex cursor-pointer items-start gap-3 rounded-lg border border-slate-200 p-3 dark:border-slate-700">
        <input type="radio" name="mode" class="mt-1 accent-brand-600" checked={keyring.storageMode === 'persistent'} onchange={() => setMode('persistent')} />
        <div><p class="font-medium">Persistent (IndexedDB)</p><p class="text-xs text-slate-500 dark:text-slate-400">Secret keys stored as OpenPGP passphrase-encrypted packets.</p></div>
      </label>

      {#if keyring.storageMode === 'persistent'}
        <div class="rounded-lg border border-slate-200 p-3 dark:border-slate-700">
          <div class="flex items-center justify-between">
            <div>
              <p class="font-medium">Vault passphrase (Argon2id + AES-256-GCM)</p>
              <p class="text-xs text-slate-500 dark:text-slate-400">Adds a second encryption layer over the whole keyring blob.</p>
            </div>
            {#if keyring.vaultEnabled}
              <button class="btn-secondary" onclick={disableVault}>Disable</button>
            {:else}
              <button class="btn-primary" onclick={() => (showVault = true)}>Enable</button>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </section>

  <section class="card p-5">
    <h3 class="mb-3 flex items-center gap-2 font-semibold"><Icon name="shield" size={18} /> Security</h3>
    <div class="space-y-3">
      <button class="btn-secondary w-full justify-start" onclick={() => { keyring.clearPassphraseCache(); toasts.success('Session passphrase cache cleared.'); }}>
        <Icon name="x" size={16} /> Clear cached passphrases
      </button>
      <button class="btn-danger w-full justify-start" onclick={() => (showWipe = true)}>
        <Icon name="trash" size={16} /> Wipe all local data
      </button>
    </div>
  </section>

  <section class="card p-5 text-sm">
    <h3 class="mb-2 flex items-center gap-2 font-semibold"><Icon name="info" size={18} /> About</h3>
    <p class="text-slate-600 dark:text-slate-300">
      enKrypt runs entirely in your browser. There are no servers, no telemetry, and no network requests at runtime.
      OpenPGP operations are performed by the pure-Rust <span class="mono">rpgp</span> library compiled to WebAssembly.
    </p>
    <p class="mt-2 mono text-xs text-slate-400">{coreVersion}</p>
  </section>
</div>

<Dialog open={showVault} title="Enable vault passphrase" size="sm" onClose={() => (showVault = false)}>
  <div class="space-y-3">
    <PassphraseInput bind:value={vaultPass} label="Vault passphrase" meter id="set-vp1" />
    <PassphraseInput bind:value={vaultPass2} label="Confirm" id="set-vp2" />
  </div>
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => (showVault = false)}>Cancel</button>
    <button class="btn-primary" onclick={enableVault}>Enable vault</button>
  {/snippet}
</Dialog>

<Dialog open={showWipe} title="Wipe all local data" size="sm" onClose={() => (showWipe = false)}>
  <p class="text-sm text-slate-600 dark:text-slate-300">
    This permanently deletes <strong>all keys</strong> and settings from this browser (IndexedDB and memory).
    Make sure you have exported anything you need. This cannot be undone.
  </p>
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => (showWipe = false)}>Cancel</button>
    <button class="btn-danger" onclick={wipe}>Wipe everything</button>
  {/snippet}
</Dialog>
