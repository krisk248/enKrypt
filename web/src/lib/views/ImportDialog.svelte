<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import DropZone from '../components/DropZone.svelte';
  import Spinner from '../components/Spinner.svelte';
  import Icon from '../components/Icon.svelte';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { readFileBytes, formatFingerprint, readClipboard } from '../util/format';
  import type { KeyInfo } from '../../wasm/enkrypt_core';

  interface Props {
    open: boolean;
    onClose: () => void;
    presetBytes?: Uint8Array | null;
  }
  let { open, onClose, presetBytes = null }: Props = $props();

  let text = $state('');
  let bytes = $state<Uint8Array | null>(null);
  let preview = $state<KeyInfo | null>(null);
  let busy = $state(false);

  const enc = new TextEncoder();

  $effect(() => {
    if (open && presetBytes) {
      bytes = presetBytes;
      void review();
    }
  });

  function resetAll() {
    text = '';
    bytes = null;
    preview = null;
  }

  async function onFiles(files: File[]) {
    bytes = await readFileBytes(files[0]);
    await review();
  }

  async function pasteClipboard() {
    try {
      text = await readClipboard();
      await reviewText();
    } catch {
      toasts.error('Clipboard access was denied.');
    }
  }

  async function reviewText() {
    if (!text.trim()) return;
    bytes = enc.encode(text);
    await review();
  }

  async function review() {
    if (!bytes) return;
    busy = true;
    try {
      preview = await keyring.inspect(bytes);
    } catch (e) {
      preview = null;
      toasts.error(`Could not read key: ${(e as Error).message}`);
    } finally {
      busy = false;
    }
  }

  async function confirmImport() {
    if (!bytes) return;
    busy = true;
    try {
      const stored = await keyring.importBytes(bytes);
      toasts.success(
        `Imported ${stored.info.primary_user_id ?? stored.info.short_id}${stored.info.has_secret ? ' (with secret key)' : ''}.`,
      );
      resetAll();
      onClose();
    } catch (e) {
      toasts.error(`Import failed: ${(e as Error).message}`);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog {open} onClose={() => { resetAll(); onClose(); }} title="Import a key" size="lg">
  {#if !preview}
    <div class="space-y-4">
      <DropZone onFiles={onFiles} icon="upload" hint="Drop a .asc / .gpg key file, or click to browse" accept=".asc,.gpg,.pgp,.key,application/pgp-keys" />
      <div class="flex items-center gap-3 text-xs text-slate-400">
        <div class="h-px flex-1 bg-slate-200 dark:bg-slate-700"></div>
        OR PASTE ARMORED TEXT
        <div class="h-px flex-1 bg-slate-200 dark:bg-slate-700"></div>
      </div>
      <textarea
        class="input h-40 resize-none font-mono text-xs"
        placeholder="-----BEGIN PGP PUBLIC KEY BLOCK-----"
        bind:value={text}
      ></textarea>
      <div class="flex gap-2">
        <button class="btn-secondary" onclick={pasteClipboard}>
          <Icon name="clipboard" size={16} /> Paste from clipboard
        </button>
        <button class="btn-primary ml-auto" onclick={reviewText} disabled={!text.trim() || busy}>
          {#if busy}<Spinner size={16} />{/if} Review
        </button>
      </div>
    </div>
  {:else}
    <div class="space-y-3">
      <div class="flex items-center gap-2 rounded-lg bg-slate-50 px-3 py-2 text-sm dark:bg-slate-800/60">
        <Icon name={preview.has_secret ? 'lock' : 'keys'} size={18} class="text-brand-600 dark:text-brand-300" />
        <span class="font-medium">{preview.has_secret ? 'Secret + public key' : 'Public key'}</span>
      </div>
      <dl class="grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1.5 text-sm">
        <dt class="text-slate-500">User ID</dt>
        <dd class="font-medium">{preview.primary_user_id ?? '—'}</dd>
        {#each preview.user_ids.slice(1) as uid}
          <dt class="text-slate-500">Also</dt>
          <dd>{uid}</dd>
        {/each}
        <dt class="text-slate-500">Fingerprint</dt>
        <dd class="mono break-all">{formatFingerprint(preview.fingerprint)}</dd>
        <dt class="text-slate-500">Algorithm</dt>
        <dd>{preview.algorithm}{preview.bits ? ` · ${preview.bits} bits` : ''}</dd>
        <dt class="text-slate-500">Subkeys</dt>
        <dd>{preview.subkeys.length}</dd>
      </dl>
    </div>
  {/if}

  {#snippet footer()}
    {#if preview}
      <button class="btn-secondary" onclick={() => (preview = null)} disabled={busy}>Back</button>
      <button class="btn-primary" onclick={confirmImport} disabled={busy}>
        {#if busy}<Spinner size={16} />{/if} Add to keyring
      </button>
    {:else}
      <button class="btn-secondary" onclick={() => { resetAll(); onClose(); }}>Cancel</button>
    {/if}
  {/snippet}
</Dialog>
