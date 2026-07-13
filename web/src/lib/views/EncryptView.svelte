<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import DropZone from '../components/DropZone.svelte';
  import Empty from '../components/Empty.svelte';
  import Spinner from '../components/Spinner.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import { cryptoApi } from '../worker/rpc';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { readFileBytes, download, formatBytes } from '../util/format';

  const MAX = 50 * 1024 * 1024;

  let file = $state<File | null>(null);
  let recipients = $state<Set<string>>(new Set());
  let alsoSign = $state(false);
  let signerFpr = $state<string | null>(null);
  let signPass = $state('');
  let armor = $state(false);
  let busy = $state(false);

  let hasKeys = $derived(keyring.keys.length > 0);

  $effect(() => {
    // Preselect the default secret key as signer.
    if (alsoSign && !signerFpr) signerFpr = keyring.defaultKeyFpr ?? keyring.secretKeys[0]?.fingerprint ?? null;
  });

  function pickFile(files: File[]) {
    if (files[0].size > MAX) return toasts.error(`File too large (max ${formatBytes(MAX)}).`);
    file = files[0];
  }

  function toggleRecipient(fpr: string) {
    const next = new Set(recipients);
    next.has(fpr) ? next.delete(fpr) : next.add(fpr);
    recipients = next;
  }

  async function run() {
    if (!file) return toasts.error('Choose a file to encrypt.');
    if (recipients.size === 0) return toasts.error('Select at least one recipient.');
    busy = true;
    try {
      const data = await readFileBytes(file);
      const recipKeys = [...recipients].map((f) => keyring.getKey(f)!.publicKey);
      let signKey: string | null = null;
      if (alsoSign) {
        const sk = signerFpr ? keyring.getKey(signerFpr) : null;
        if (!sk?.secretKey) throw new Error('Select a valid signing key.');
        signKey = sk.secretKey;
      }
      const out = await cryptoApi.encrypt(data, recipKeys, signKey, alsoSign ? signPass : null, armor);
      const ext = armor ? '.asc' : '.gpg';
      download(`${file.name}${ext}`, out, armor ? 'application/pgp-encrypted' : 'application/octet-stream');
      toasts.success(`Encrypted to ${recipients.size} recipient${recipients.size > 1 ? 's' : ''}.`);
      signPass = '';
    } catch (e) {
      toasts.error(`Encryption failed: ${(e as Error).message}`);
    } finally {
      busy = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl p-6">
  <h2 class="text-lg font-bold">Encrypt a file</h2>
  <p class="mb-5 text-xs text-slate-500 dark:text-slate-400">Everything happens locally. The file never leaves your device.</p>

  {#if !hasKeys}
    <Empty icon="keys" title="No recipient keys" description="Import or generate a public key before encrypting." />
  {:else}
    <div class="space-y-5">
      {#if file}
        <div class="card flex items-center gap-3 px-4 py-3">
          <Icon name="file" size={22} class="text-brand-600 dark:text-brand-300" />
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-medium">{file.name}</p>
            <p class="text-xs text-slate-500">{formatBytes(file.size)}</p>
          </div>
          <button class="btn-ghost p-1.5" onclick={() => (file = null)} aria-label="Remove file"><Icon name="x" size={16} /></button>
        </div>
      {:else}
        <DropZone onFiles={pickFile} icon="lock" hint="Drop a file to encrypt, or click to browse" />
      {/if}

      <div>
        <span class="label">Recipients</span>
        <div class="card max-h-56 divide-y divide-slate-100 overflow-y-auto dark:divide-slate-800">
          {#each keyring.keys as k (k.fingerprint)}
            <label class="flex cursor-pointer items-center gap-3 px-3.5 py-2.5 hover:bg-slate-50 dark:hover:bg-slate-800/50">
              <input type="checkbox" class="h-4 w-4 rounded accent-brand-600" checked={recipients.has(k.fingerprint)} onchange={() => toggleRecipient(k.fingerprint)} />
              <Icon name="keys" size={16} class="text-slate-400" />
              <div class="min-w-0 flex-1">
                <p class="truncate text-sm font-medium">{k.info.primary_user_id ?? k.info.short_id}</p>
                <p class="mono text-xs text-slate-500">{k.info.key_id}</p>
              </div>
            </label>
          {/each}
        </div>
      </div>

      <div class="card p-4">
        <label class="flex items-center gap-2.5 text-sm font-medium">
          <input type="checkbox" class="h-4 w-4 rounded accent-brand-600" bind:checked={alsoSign} disabled={keyring.secretKeys.length === 0} />
          Also sign with my key
          {#if keyring.secretKeys.length === 0}<span class="text-xs font-normal text-slate-400">(no secret key available)</span>{/if}
        </label>
        {#if alsoSign}
          <div class="mt-3 grid gap-3 sm:grid-cols-2">
            <div>
              <label class="label" for="enc-signer">Signing key</label>
              <select id="enc-signer" class="input" bind:value={signerFpr}>
                {#each keyring.secretKeys as k}
                  <option value={k.fingerprint}>{k.info.primary_user_id ?? k.info.short_id}</option>
                {/each}
              </select>
            </div>
            <PassphraseInput bind:value={signPass} label="Key passphrase" id="enc-pass" />
          </div>
        {/if}
      </div>

      <div class="flex items-center justify-between">
        <label class="flex items-center gap-2.5 text-sm">
          <input type="checkbox" class="h-4 w-4 rounded accent-brand-600" bind:checked={armor} />
          ASCII-armored output (.asc)
        </label>
        <button class="btn-primary" onclick={run} disabled={busy}>
          {#if busy}<Spinner size={16} />Encrypting…{:else}<Icon name="lock" size={16} />Encrypt &amp; download{/if}
        </button>
      </div>
    </div>
  {/if}
</div>
