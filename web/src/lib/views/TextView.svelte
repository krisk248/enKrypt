<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import Spinner from '../components/Spinner.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import SignatureBadge from '../components/SignatureBadge.svelte';
  import { cryptoApi } from '../worker/rpc';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { copyToClipboard, readClipboard } from '../util/format';
  import type { SignatureInfo } from '../../wasm/enkrypt_core';

  type Mode = 'encrypt' | 'decrypt' | 'sign' | 'verify';
  let mode = $state<Mode>('encrypt');

  let input = $state('');
  let output = $state('');
  let sigs = $state<SignatureInfo[]>([]);
  let passphrase = $state('');
  let recipients = $state<Set<string>>(new Set());
  let signerFpr = $state<string | null>(null);
  let busy = $state(false);

  const enc = new TextEncoder();
  const dec = new TextDecoder();

  const modes: { id: Mode; label: string }[] = [
    { id: 'encrypt', label: 'Encrypt' },
    { id: 'decrypt', label: 'Decrypt' },
    { id: 'sign', label: 'Clear-sign' },
    { id: 'verify', label: 'Verify' },
  ];

  $effect(() => {
    if ((mode === 'sign') && !signerFpr)
      signerFpr = keyring.defaultKeyFpr ?? keyring.secretKeys[0]?.fingerprint ?? null;
  });

  function toggleRecipient(fpr: string) {
    const next = new Set(recipients);
    next.has(fpr) ? next.delete(fpr) : next.add(fpr);
    recipients = next;
  }

  async function run() {
    busy = true;
    sigs = [];
    output = '';
    try {
      if (mode === 'encrypt') {
        if (recipients.size === 0) throw new Error('Select at least one recipient.');
        const recipKeys = [...recipients].map((f) => keyring.getKey(f)!.publicKey);
        const out = await cryptoApi.encrypt(enc.encode(input), recipKeys, null, null, true);
        output = dec.decode(out);
      } else if (mode === 'decrypt') {
        const res = await cryptoApi.decrypt(enc.encode(input), keyring.secretKeysArmored(), passphrase, keyring.publicKeysArmored());
        output = dec.decode(res.data);
        sigs = res.signatures;
      } else if (mode === 'sign') {
        const sk = signerFpr ? keyring.getKey(signerFpr) : null;
        if (!sk?.secretKey) throw new Error('Select a signing key.');
        output = await cryptoApi.signCleartext(input, sk.secretKey, passphrase);
      } else {
        const res = await cryptoApi.verifyCleartext(input, keyring.publicKeysArmored());
        output = res.text;
        sigs = res.signatures;
      }
      toasts.success('Done.');
    } catch (e) {
      toasts.error((e as Error).message);
    } finally {
      busy = false;
    }
  }

  async function paste() {
    try { input = await readClipboard(); } catch { toasts.error('Clipboard access denied.'); }
  }
  async function copyOut() {
    await copyToClipboard(output);
    toasts.success('Copied to clipboard.');
  }
</script>

<div class="mx-auto max-w-4xl p-6">
  <h2 class="text-lg font-bold">Text mode</h2>
  <p class="mb-4 text-xs text-slate-500 dark:text-slate-400">Encrypt, decrypt, clear-sign or verify text — like WinPT's clipboard functions.</p>

  <div class="mb-4 inline-flex flex-wrap gap-0.5 rounded-lg border border-slate-200 p-0.5 dark:border-slate-700">
    {#each modes as m}
      <button class="rounded-md px-4 py-1.5 text-sm font-medium {mode === m.id ? 'bg-brand-600 text-white' : 'text-slate-600 dark:text-slate-300'}" onclick={() => { mode = m.id; output = ''; sigs = []; }}>{m.label}</button>
    {/each}
  </div>

  <div class="grid gap-4 lg:grid-cols-2">
    <div class="flex flex-col">
      <div class="mb-1 flex items-center justify-between">
        <span class="label mb-0">Input</span>
        <button class="btn-ghost px-2 py-1 text-xs" onclick={paste}><Icon name="clipboard" size={14} /> Paste</button>
      </div>
      <textarea class="input h-72 resize-none font-mono text-xs" bind:value={input} placeholder={mode === 'encrypt' || mode === 'sign' ? 'Type or paste text…' : '-----BEGIN PGP MESSAGE-----'}></textarea>
    </div>
    <div class="flex flex-col">
      <div class="mb-1 flex items-center justify-between">
        <span class="label mb-0">Output</span>
        <button class="btn-ghost px-2 py-1 text-xs" onclick={copyOut} disabled={!output}><Icon name="copy" size={14} /> Copy</button>
      </div>
      <textarea class="input h-72 resize-none bg-slate-50 font-mono text-xs dark:bg-slate-950/50" readonly bind:value={output} placeholder="Result appears here…"></textarea>
    </div>
  </div>

  {#if mode === 'encrypt'}
    <div class="mt-4">
      <span class="label">Recipients</span>
      <div class="card flex flex-wrap gap-2 p-3">
        {#each keyring.keys as k}
          <button class="chip border {recipients.has(k.fingerprint) ? 'border-brand-500 bg-brand-100 text-brand-700 dark:bg-brand-900/60 dark:text-brand-200' : 'border-slate-200 dark:border-slate-700'}" onclick={() => toggleRecipient(k.fingerprint)}>
            {#if recipients.has(k.fingerprint)}<Icon name="check" size={12} />{/if}
            {k.info.primary_user_id ?? k.info.short_id}
          </button>
        {/each}
        {#if keyring.keys.length === 0}<span class="text-xs text-slate-400">No keys available.</span>{/if}
      </div>
    </div>
  {/if}

  {#if mode === 'sign'}
    <div class="mt-4 grid gap-3 sm:grid-cols-2">
      <div>
        <label class="label" for="tx-signer">Signing key</label>
        <select id="tx-signer" class="input" bind:value={signerFpr}>
          {#each keyring.secretKeys as k}<option value={k.fingerprint}>{k.info.primary_user_id ?? k.info.short_id}</option>{/each}
        </select>
      </div>
      <PassphraseInput bind:value={passphrase} label="Key passphrase" id="tx-pass1" />
    </div>
  {/if}

  {#if mode === 'decrypt'}
    <div class="mt-4 max-w-sm"><PassphraseInput bind:value={passphrase} label="Key passphrase" id="tx-pass2" /></div>
  {/if}

  {#if sigs.length}
    <div class="mt-4 space-y-2">
      {#each sigs as sig}<SignatureBadge {sig} />{/each}
    </div>
  {/if}

  <div class="mt-5">
    <button class="btn-primary" onclick={run} disabled={busy || !input.trim()}>
      {#if busy}<Spinner size={16} />Working…{:else}<Icon name={mode === 'verify' ? 'shield-check' : mode === 'decrypt' ? 'unlock' : mode === 'sign' ? 'sign' : 'lock'} size={16} />{modes.find((m) => m.id === mode)?.label}{/if}
    </button>
  </div>
</div>
