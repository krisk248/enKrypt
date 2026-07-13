<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import Empty from '../components/Empty.svelte';
  import KeyGenDialog from './KeyGenDialog.svelte';
  import ImportDialog from './ImportDialog.svelte';
  import KeyPropertiesDialog from './KeyPropertiesDialog.svelte';
  import RevokeDialog from './RevokeDialog.svelte';
  import Dialog from '../components/Dialog.svelte';
  import { keyring } from '../stores/keyring.svelte';
  import { nav } from '../stores/nav.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { shortId, formatDate, formatFingerprint, download, copyToClipboard } from '../util/format';
  import type { StoredKey } from '../db/idb';

  let query = $state('');
  let selectedFpr = $state<string | null>(null);
  let showGen = $state(false);
  let showImport = $state(false);
  let propsKey = $state<StoredKey | null>(null);
  let revokeKey = $state<StoredKey | null>(null);
  let deleteKey = $state<StoredKey | null>(null);

  let menu = $state<{ x: number; y: number; key: StoredKey } | null>(null);

  let filtered = $derived(
    keyring.keys.filter((k) => {
      if (!query.trim()) return true;
      const q = query.toLowerCase();
      return (
        (k.info.primary_user_id ?? '').toLowerCase().includes(q) ||
        k.info.user_ids.some((u) => u.toLowerCase().includes(q)) ||
        k.info.fingerprint.toLowerCase().includes(q) ||
        k.info.key_id.toLowerCase().includes(q)
      );
    }),
  );

  function validity(k: StoredKey): { label: string; cls: string } {
    if (k.revoked) return { label: 'Revoked', cls: 'text-red-600 dark:text-red-400' };
    if (k.expiryOverride && k.expiryOverride * 1000 < Date.now())
      return { label: 'Expired', cls: 'text-amber-600 dark:text-amber-400' };
    return { label: 'Valid', cls: 'text-green-600 dark:text-green-400' };
  }

  function openMenu(e: MouseEvent, key: StoredKey) {
    e.preventDefault();
    selectedFpr = key.fingerprint;
    menu = { x: e.clientX, y: e.clientY, key };
  }

  async function copyPublic(k: StoredKey) {
    await copyToClipboard(k.publicKey);
    toasts.success('Public key copied to clipboard.');
  }
  function exportPublic(k: StoredKey) {
    download(`${k.info.short_id}-public.asc`, k.publicKey, 'application/pgp-keys');
    toasts.success('Public key exported.');
  }
  function exportSecret(k: StoredKey) {
    if (!k.secretKey) return;
    download(`${k.info.short_id}-secret.asc`, k.secretKey, 'application/pgp-keys');
    toasts.success('Secret key exported (passphrase-encrypted).');
  }
  async function setDefault(k: StoredKey) {
    await keyring.setDefault(k.fingerprint);
    toasts.success('Default key updated.');
  }
  async function confirmDelete() {
    if (!deleteKey) return;
    await keyring.remove(deleteKey.fingerprint);
    toasts.info('Key removed from keyring.');
    if (selectedFpr === deleteKey.fingerprint) selectedFpr = null;
    deleteKey = null;
  }

  function onKeydown(e: KeyboardEvent) {
    if (!selectedFpr) return;
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
    const k = keyring.getKey(selectedFpr);
    if (!k) return;
    if (e.key === 'Delete') {
      e.preventDefault();
      deleteKey = k;
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
      e.preventDefault();
      void copyPublic(k);
    }
  }
</script>

<svelte:window onkeydown={onKeydown} onclick={() => (menu = null)} />

<div class="flex h-full flex-col">
  <header class="flex items-center gap-3 border-b border-slate-200 px-6 py-4 dark:border-slate-800">
    <div>
      <h2 class="text-lg font-bold">Key Manager</h2>
      <p class="text-xs text-slate-500 dark:text-slate-400">Manage your OpenPGP keys — right-click a key for actions.</p>
    </div>
    <div class="ml-auto flex items-center gap-2">
      <div class="relative">
        <Icon name="search" size={16} class="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
        <input class="input w-56 pl-8" placeholder="Search keys…" bind:value={query} />
      </div>
      <button class="btn-secondary" onclick={() => (showImport = true)}>
        <Icon name="upload" size={16} /> Import
      </button>
      <button class="btn-primary" onclick={() => (showGen = true)}>
        <Icon name="plus" size={16} /> Generate
      </button>
    </div>
  </header>

  <div class="min-h-0 flex-1 overflow-auto p-6">
    {#if keyring.keys.length === 0}
      <Empty icon="keys" title="No keys yet" description="Generate a new key pair or import an existing one to get started.">
        <div class="flex justify-center gap-2">
          <button class="btn-secondary" onclick={() => (showImport = true)}><Icon name="upload" size={16} /> Import key</button>
          <button class="btn-primary" onclick={() => (showGen = true)}><Icon name="plus" size={16} /> Generate key</button>
        </div>
      </Empty>
    {:else if filtered.length === 0}
      <Empty icon="search" title="No matches" description="No keys match your search." />
    {:else}
      <div class="card overflow-hidden">
        <table class="w-full text-sm">
          <thead class="border-b border-slate-200 bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500 dark:border-slate-800 dark:bg-slate-800/40 dark:text-slate-400">
            <tr>
              <th class="px-4 py-2.5 font-semibold">User ID</th>
              <th class="px-4 py-2.5 font-semibold">Key ID</th>
              <th class="px-4 py-2.5 font-semibold">Type</th>
              <th class="px-4 py-2.5 font-semibold">Algorithm</th>
              <th class="px-4 py-2.5 font-semibold">Validity</th>
              <th class="px-4 py-2.5 font-semibold">Created</th>
              <th class="px-2 py-2.5"></th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as k (k.fingerprint)}
              {@const v = validity(k)}
              <tr
                class="cursor-default border-b border-slate-100 transition-colors last:border-0 dark:border-slate-800/70
                  {selectedFpr === k.fingerprint ? 'bg-brand-50 dark:bg-brand-950/40' : 'hover:bg-slate-50 dark:hover:bg-slate-800/40'}"
                onclick={() => (selectedFpr = k.fingerprint)}
                ondblclick={() => (propsKey = k)}
                oncontextmenu={(e) => openMenu(e, k)}
              >
                <td class="px-4 py-2.5">
                  <div class="flex items-center gap-2.5">
                    <Icon name={k.secretKey ? 'lock' : 'keys'} size={17} class="shrink-0 text-brand-600 dark:text-brand-300" />
                    <div class="min-w-0">
                      <p class="truncate font-medium">
                        {k.info.primary_user_id ?? k.info.short_id}
                        {#if k.fingerprint === keyring.defaultKeyFpr}
                          <Icon name="star" size={12} fill class="ml-1 inline text-amber-500" />
                        {/if}
                      </p>
                    </div>
                  </div>
                </td>
                <td class="px-4 py-2.5">
                  <span class="mono" title={formatFingerprint(k.info.fingerprint)}>{shortId(k.info.key_id)}</span>
                </td>
                <td class="px-4 py-2.5">
                  <span class="chip {k.secretKey ? 'bg-brand-100 text-brand-700 dark:bg-brand-900/60 dark:text-brand-200' : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300'}">
                    {k.secretKey ? 'pub + sec' : 'pub'}
                  </span>
                </td>
                <td class="px-4 py-2.5 text-slate-600 dark:text-slate-300">
                  {k.info.algorithm}{k.info.bits ? ` ${k.info.bits}` : ''}
                </td>
                <td class="px-4 py-2.5"><span class="font-medium {v.cls}">{v.label}</span></td>
                <td class="px-4 py-2.5 text-slate-500 dark:text-slate-400">{formatDate(k.info.created_at)}</td>
                <td class="px-2 py-2.5">
                  <button class="btn-ghost p-1.5" onclick={(e) => { e.stopPropagation(); openMenu(e, k); }} aria-label="Actions">
                    <Icon name="dots" size={16} />
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <p class="mt-3 text-xs text-slate-400">
        Tip: <kbd class="rounded border border-slate-300 px-1 dark:border-slate-600">Del</kbd> to remove a selected key,
        <kbd class="rounded border border-slate-300 px-1 dark:border-slate-600">Ctrl+C</kbd> to copy its public key.
      </p>
    {/if}
  </div>
</div>

<!-- Context menu -->
{#if menu}
  {@const k = menu.key}
  <div
    class="card fixed z-50 w-56 overflow-hidden py-1 text-sm shadow-xl animate-fade-in"
    style="left: {Math.min(menu.x, window.innerWidth - 240)}px; top: {Math.min(menu.y, window.innerHeight - 320)}px"
    role="menu"
  >
    {#snippet item(label: string, icon: string, action: () => void, danger = false)}
      <button
        class="flex w-full items-center gap-2.5 px-3 py-1.5 text-left hover:bg-slate-100 dark:hover:bg-slate-800 {danger ? 'text-red-600 dark:text-red-400' : ''}"
        onclick={() => { action(); menu = null; }}
        role="menuitem"
      >
        <Icon name={icon} size={16} class="shrink-0 opacity-70" />
        {label}
      </button>
    {/snippet}

    {@render item('Copy public key', 'copy', () => copyPublic(k))}
    {@render item('Export public key', 'download', () => exportPublic(k))}
    {#if k.secretKey}
      {@render item('Export secret key', 'download', () => exportSecret(k))}
      <div class="my-1 h-px bg-slate-200 dark:bg-slate-700"></div>
      {@render item('Sign a file with this key', 'sign', () => nav.signWith(k.fingerprint))}
      {#if k.fingerprint !== keyring.defaultKeyFpr}
        {@render item('Set as default key', 'star', () => setDefault(k))}
      {/if}
      {@render item('Revoke…', 'revoke', () => (revokeKey = k), true)}
    {/if}
    <div class="my-1 h-px bg-slate-200 dark:bg-slate-700"></div>
    {@render item('Properties', 'info', () => (propsKey = k))}
    {@render item('Delete', 'trash', () => (deleteKey = k), true)}
  </div>
{/if}

<KeyGenDialog open={showGen} onClose={() => (showGen = false)} />
<ImportDialog open={showImport} onClose={() => (showImport = false)} />
<KeyPropertiesDialog open={!!propsKey} keyItem={propsKey} onClose={() => (propsKey = null)} />
<RevokeDialog open={!!revokeKey} keyItem={revokeKey} onClose={() => (revokeKey = null)} />

<Dialog open={!!deleteKey} title="Delete key" size="sm" onClose={() => (deleteKey = null)}>
  <p class="text-sm text-slate-600 dark:text-slate-300">
    Remove <strong>{deleteKey?.info.primary_user_id ?? deleteKey?.info.short_id}</strong> from your keyring?
    {#if deleteKey?.secretKey}
      <span class="mt-2 block rounded-lg bg-red-50 px-3 py-2 text-red-700 dark:bg-red-950/50 dark:text-red-300">
        This includes the <strong>secret key</strong>. Make sure you have a backup — this cannot be undone.
      </span>
    {/if}
  </p>
  {#snippet footer()}
    <button class="btn-secondary" onclick={() => (deleteKey = null)}>Cancel</button>
    <button class="btn-danger" onclick={confirmDelete}>Delete</button>
  {/snippet}
</Dialog>
