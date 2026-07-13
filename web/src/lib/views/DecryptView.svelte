<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import DropZone from '../components/DropZone.svelte';
  import Empty from '../components/Empty.svelte';
  import Spinner from '../components/Spinner.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import SignatureBadge from '../components/SignatureBadge.svelte';
  import { cryptoApi, CryptoError } from '../worker/rpc';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { readFileBytes, download, formatBytes, stripEncryptedExt } from '../util/format';
  import type { DecryptResult } from '../../wasm/enkrypt_core';

  let file = $state<File | null>(null);
  let passphrase = $state('');
  let remember = $state(false);
  let busy = $state(false);
  let result = $state<DecryptResult | null>(null);
  let outName = $state('decrypted.bin');

  let hasSecret = $derived(keyring.secretKeys.length > 0);

  function pickFile(files: File[]) {
    file = files[0];
    result = null;
    // Pre-fill a cached passphrase for the default key if present.
    const cached = keyring.defaultKeyFpr ? keyring.cachedPassphrase(keyring.defaultKeyFpr) : undefined;
    if (cached) passphrase = cached;
  }

  async function run() {
    if (!file) return toasts.error('Choose a file to decrypt.');
    busy = true;
    result = null;
    try {
      const data = await readFileBytes(file);
      const res = await cryptoApi.decrypt(
        data,
        keyring.secretKeysArmored(),
        passphrase,
        keyring.publicKeysArmored(),
      );
      result = res;
      outName = stripEncryptedExt(file.name);
      if (remember && keyring.defaultKeyFpr) keyring.cachePassphrase(keyring.defaultKeyFpr, passphrase);
      toasts.success('Decrypted successfully.');
    } catch (e) {
      const code = e instanceof CryptoError ? e.code : '';
      if (code === 'wrong_passphrase') toasts.error('Wrong passphrase.');
      else if (code === 'no_matching_key') toasts.error('No matching secret key for this message.');
      else toasts.error(`Decryption failed: ${(e as Error).message}`);
    } finally {
      busy = false;
    }
  }

  function saveResult() {
    if (result) download(outName, result.data);
  }
</script>

<div class="mx-auto max-w-3xl p-6">
  <h2 class="text-lg font-bold">Decrypt a file</h2>
  <p class="mb-5 text-xs text-slate-500 dark:text-slate-400">The right secret key is detected automatically from the message.</p>

  {#if !hasSecret}
    <Empty icon="lock" title="No secret keys" description="Import or generate a secret key to decrypt messages." />
  {:else}
    <div class="space-y-5">
      {#if file}
        <div class="card flex items-center gap-3 px-4 py-3">
          <Icon name="file" size={22} class="text-brand-600 dark:text-brand-300" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium">{file.name}</p>
            <p class="text-xs text-slate-500">{formatBytes(file.size)}</p>
          </div>
          <button class="btn-ghost p-1.5" onclick={() => { file = null; result = null; }} aria-label="Remove"><Icon name="x" size={16} /></button>
        </div>
      {:else}
        <DropZone onFiles={pickFile} icon="unlock" hint="Drop an encrypted .gpg / .asc file, or click to browse" accept=".gpg,.pgp,.asc" />
      {/if}

      <div class="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-end">
        <PassphraseInput bind:value={passphrase} label="Key passphrase" id="dec-pass" />
        <button class="btn-primary h-[38px]" onclick={run} disabled={busy || !file}>
          {#if busy}<Spinner size={16} />Decrypting…{:else}<Icon name="unlock" size={16} />Decrypt{/if}
        </button>
      </div>
      <label class="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400">
        <input type="checkbox" class="h-3.5 w-3.5 rounded accent-brand-600" bind:checked={remember} />
        Remember passphrase for this session (kept in memory only)
      </label>

      {#if result}
        <div class="card space-y-3 p-4 animate-fade-in">
          <div class="flex items-center gap-2 text-green-700 dark:text-green-300">
            <Icon name="check" size={18} /> <span class="font-semibold">Decrypted {formatBytes(result.data.length)}</span>
          </div>
          {#each result.signatures as sig}
            <SignatureBadge {sig} />
          {/each}
          {#if result.signatures.length === 0}
            <p class="text-xs text-slate-500 dark:text-slate-400">This message was not signed.</p>
          {/if}
          <div class="flex items-center gap-2">
            <input class="input flex-1" bind:value={outName} />
            <button class="btn-primary" onclick={saveResult}><Icon name="download" size={16} /> Save</button>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
