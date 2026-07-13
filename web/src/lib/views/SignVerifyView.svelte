<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import DropZone from '../components/DropZone.svelte';
  import Spinner from '../components/Spinner.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import SignatureBadge from '../components/SignatureBadge.svelte';
  import Empty from '../components/Empty.svelte';
  import { cryptoApi } from '../worker/rpc';
  import { keyring } from '../stores/keyring.svelte';
  import { nav } from '../stores/nav.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { readFileBytes, download, formatBytes } from '../util/format';
  import type { VerifyResult } from '../../wasm/enkrypt_core';

  let tab = $state<'sign' | 'verify'>('sign');

  // --- sign ---
  let signFile = $state<File | null>(null);
  let signerFpr = $state<string | null>(null);
  let signPass = $state('');
  let armor = $state(true);
  let signing = $state(false);

  // --- verify ---
  let vFile = $state<File | null>(null);
  let vSig = $state<File | null>(null);
  let verifying = $state(false);
  let vResult = $state<VerifyResult | null>(null);

  $effect(() => {
    if (nav.signerFpr) {
      tab = 'sign';
      signerFpr = nav.signerFpr;
      nav.signerFpr = null;
    }
    if (!signerFpr) signerFpr = keyring.defaultKeyFpr ?? keyring.secretKeys[0]?.fingerprint ?? null;
  });

  async function doSign() {
    if (!signFile) return toasts.error('Choose a file to sign.');
    const sk = signerFpr ? keyring.getKey(signerFpr) : null;
    if (!sk?.secretKey) return toasts.error('Select a signing key.');
    signing = true;
    try {
      const data = await readFileBytes(signFile);
      const sig = await cryptoApi.signDetached(data, sk.secretKey, signPass, armor);
      download(`${signFile.name}${armor ? '.asc' : '.sig'}`, sig, 'application/pgp-signature');
      toasts.success('Detached signature created.');
      signPass = '';
    } catch (e) {
      toasts.error(`Signing failed: ${(e as Error).message}`);
    } finally {
      signing = false;
    }
  }

  async function doVerify() {
    if (!vFile || !vSig) return toasts.error('Provide both the file and its signature.');
    verifying = true;
    vResult = null;
    try {
      const data = await readFileBytes(vFile);
      const sig = await readFileBytes(vSig);
      vResult = await cryptoApi.verifyDetached(data, sig, keyring.publicKeysArmored());
    } catch (e) {
      toasts.error(`Verification failed: ${(e as Error).message}`);
    } finally {
      verifying = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl p-6">
  <h2 class="text-lg font-bold">Sign &amp; Verify</h2>
  <p class="mb-4 text-xs text-slate-500 dark:text-slate-400">Create and check detached signatures (.sig / .asc).</p>

  <div class="mb-5 inline-flex rounded-lg border border-slate-200 p-0.5 dark:border-slate-700">
    <button class="rounded-md px-4 py-1.5 text-sm font-medium {tab === 'sign' ? 'bg-brand-600 text-white' : 'text-slate-600 dark:text-slate-300'}" onclick={() => (tab = 'sign')}>Sign</button>
    <button class="rounded-md px-4 py-1.5 text-sm font-medium {tab === 'verify' ? 'bg-brand-600 text-white' : 'text-slate-600 dark:text-slate-300'}" onclick={() => (tab = 'verify')}>Verify</button>
  </div>

  {#if tab === 'sign'}
    {#if keyring.secretKeys.length === 0}
      <Empty icon="sign" title="No secret keys" description="Import or generate a secret key to sign files." />
    {:else}
      <div class="space-y-5">
        {#if signFile}
          <div class="card flex items-center gap-3 px-4 py-3">
            <Icon name="file" size={22} class="text-brand-600 dark:text-brand-300" />
            <div class="min-w-0 flex-1"><p class="truncate text-sm font-medium">{signFile.name}</p><p class="text-xs text-slate-500">{formatBytes(signFile.size)}</p></div>
            <button class="btn-ghost p-1.5" onclick={() => (signFile = null)} aria-label="Remove"><Icon name="x" size={16} /></button>
          </div>
        {:else}
          <DropZone onFiles={(f) => (signFile = f[0])} icon="sign" hint="Drop a file to sign, or click to browse" />
        {/if}
        <div class="grid gap-3 sm:grid-cols-2">
          <div>
            <label class="label" for="sv-signer">Signing key</label>
            <select id="sv-signer" class="input" bind:value={signerFpr}>
              {#each keyring.secretKeys as k}<option value={k.fingerprint}>{k.info.primary_user_id ?? k.info.short_id}</option>{/each}
            </select>
          </div>
          <PassphraseInput bind:value={signPass} label="Key passphrase" id="sv-pass" />
        </div>
        <div class="flex items-center justify-between">
          <label class="flex items-center gap-2 text-sm"><input type="checkbox" class="h-4 w-4 rounded accent-brand-600" bind:checked={armor} /> Armored (.asc)</label>
          <button class="btn-primary" onclick={doSign} disabled={signing}>
            {#if signing}<Spinner size={16} />Signing…{:else}<Icon name="sign" size={16} />Create signature{/if}
          </button>
        </div>
      </div>
    {/if}
  {:else}
    <div class="space-y-4">
      <div class="grid gap-4 sm:grid-cols-2">
        <div>
          <span class="label">File</span>
          {#if vFile}
            <div class="card flex items-center gap-2 px-3 py-2.5 text-sm"><Icon name="file" size={18} /><span class="min-w-0 flex-1 truncate">{vFile.name}</span><button onclick={() => (vFile = null)} aria-label="Remove"><Icon name="x" size={15} /></button></div>
          {:else}
            <DropZone onFiles={(f) => (vFile = f[0])} icon="file" hint="Original file" />
          {/if}
        </div>
        <div>
          <span class="label">Signature</span>
          {#if vSig}
            <div class="card flex items-center gap-2 px-3 py-2.5 text-sm"><Icon name="sign" size={18} /><span class="min-w-0 flex-1 truncate">{vSig.name}</span><button onclick={() => (vSig = null)} aria-label="Remove"><Icon name="x" size={15} /></button></div>
          {:else}
            <DropZone onFiles={(f) => (vSig = f[0])} icon="sign" hint=".sig / .asc" accept=".sig,.asc,.pgp" />
          {/if}
        </div>
      </div>
      <button class="btn-primary" onclick={doVerify} disabled={verifying || !vFile || !vSig}>
        {#if verifying}<Spinner size={16} />Verifying…{:else}<Icon name="shield-check" size={16} />Verify signature{/if}
      </button>
      {#if vResult}
        <div class="space-y-2 animate-fade-in">
          {#each vResult.signatures as sig}<SignatureBadge {sig} />{/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
