<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../components/Icon.svelte';
  import { formatFingerprint, formatDate, copyToClipboard } from '../util/format';
  import { toasts } from '../stores/toast.svelte';
  import type { StoredKey } from '../db/idb';

  interface Props {
    open: boolean;
    onClose: () => void;
    keyItem: StoredKey | null;
  }
  let { open, onClose, keyItem }: Props = $props();

  async function copyFpr() {
    if (!keyItem) return;
    await copyToClipboard(keyItem.info.fingerprint);
    toasts.success('Fingerprint copied.');
  }
</script>

<Dialog {open} {onClose} title="Key properties" size="lg">
  {#if keyItem}
    {@const info = keyItem.info}
    <div class="space-y-5">
      <div class="flex items-start gap-3">
        <div class="grid h-11 w-11 shrink-0 place-items-center rounded-xl bg-brand-100 text-brand-600 dark:bg-brand-900/50 dark:text-brand-300">
          <Icon name={keyItem.secretKey ? 'lock' : 'keys'} size={22} />
        </div>
        <div class="min-w-0">
          <p class="font-semibold">{info.primary_user_id ?? info.short_id}</p>
          <p class="text-sm text-slate-500 dark:text-slate-400">
            {keyItem.secretKey ? 'Public + secret key pair' : 'Public key only'}
            {#if keyItem.revoked}· <span class="text-red-600">revoked</span>{/if}
          </p>
        </div>
      </div>

      {#if info.user_ids.length > 1}
        <section>
          <h4 class="label">User IDs</h4>
          <ul class="space-y-1 text-sm">
            {#each info.user_ids as uid, i}
              <li class="flex items-center gap-2">
                {#if i === 0}<Icon name="star" size={13} fill class="text-amber-500" />{:else}<span class="w-[13px]"></span>{/if}
                {uid}
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      <section>
        <h4 class="label">Primary key</h4>
        <dl class="grid grid-cols-[9rem_1fr] gap-x-3 gap-y-1.5 text-sm">
          <dt class="text-slate-500">Fingerprint</dt>
          <dd class="flex items-center gap-2">
            <span class="mono break-all">{formatFingerprint(info.fingerprint)}</span>
            <button class="text-slate-400 hover:text-brand-600" onclick={copyFpr} aria-label="Copy fingerprint">
              <Icon name="copy" size={15} />
            </button>
          </dd>
          <dt class="text-slate-500">Key ID</dt>
          <dd class="mono">{info.key_id}</dd>
          <dt class="text-slate-500">Algorithm</dt>
          <dd>{info.algorithm}{info.bits ? ` · ${info.bits} bits` : ''}</dd>
          <dt class="text-slate-500">Created</dt>
          <dd>{formatDate(info.created_at)}</dd>
          {#if keyItem.expiryOverride}
            <dt class="text-slate-500">Expires</dt>
            <dd>{formatDate(keyItem.expiryOverride)}</dd>
          {/if}
          <dt class="text-slate-500">Capabilities</dt>
          <dd class="flex gap-1.5">
            {#if info.can_sign}<span class="chip bg-brand-100 text-brand-700 dark:bg-brand-900/60 dark:text-brand-200">Sign · Certify</span>{/if}
          </dd>
        </dl>
      </section>

      {#if info.subkeys.length}
        <section>
          <h4 class="label">Subkeys</h4>
          <div class="space-y-2">
            {#each info.subkeys as sub}
              <div class="rounded-lg border border-slate-200 p-2.5 text-sm dark:border-slate-700">
                <div class="flex items-center justify-between">
                  <span class="mono">{sub.key_id}</span>
                  <div class="flex gap-1.5">
                    {#if sub.can_encrypt}<span class="chip bg-green-100 text-green-700 dark:bg-green-900/60 dark:text-green-200">Encrypt</span>{/if}
                    {#if sub.can_sign}<span class="chip bg-brand-100 text-brand-700 dark:bg-brand-900/60 dark:text-brand-200">Sign</span>{/if}
                  </div>
                </div>
                <p class="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
                  {sub.algorithm}{sub.bits ? ` · ${sub.bits} bits` : ''} · created {formatDate(sub.created_at)}
                </p>
              </div>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  {/if}

  {#snippet footer()}
    <button class="btn-secondary" onclick={onClose}>Close</button>
  {/snippet}
</Dialog>
