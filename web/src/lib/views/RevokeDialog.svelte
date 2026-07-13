<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import Spinner from '../components/Spinner.svelte';
  import Icon from '../components/Icon.svelte';
  import { cryptoApi } from '../worker/rpc';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import { download } from '../util/format';
  import type { StoredKey } from '../db/idb';

  interface Props {
    open: boolean;
    onClose: () => void;
    keyItem: StoredKey | null;
  }
  let { open, onClose, keyItem }: Props = $props();

  let passphrase = $state('');
  let reasonCode = $state(1);
  let reason = $state('');
  let busy = $state(false);

  const enc = new TextEncoder();

  async function revoke() {
    if (!keyItem?.secretKey) return;
    busy = true;
    try {
      const cert = await cryptoApi.generateRevocation(
        enc.encode(keyItem.secretKey),
        passphrase,
        reasonCode,
        reason,
      );
      download(`${keyItem.info.short_id}-revocation.asc`, cert, 'application/pgp-signature');
      await keyring.markRevoked(keyItem.fingerprint);
      toasts.success('Revocation certificate generated and downloaded.');
      passphrase = reason = '';
      onClose();
    } catch (e) {
      toasts.error(`Could not create revocation: ${(e as Error).message}`);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog {open} {onClose} title="Generate revocation certificate" size="md">
  <div class="space-y-4">
    <div class="flex items-start gap-2.5 rounded-lg border border-amber-500/30 bg-amber-50 px-3 py-2.5 text-sm text-amber-800 dark:bg-amber-950/50 dark:text-amber-200">
      <Icon name="warn" size={18} class="mt-0.5 shrink-0" />
      <p>Store this certificate somewhere safe. Publishing it permanently marks the key as revoked.</p>
    </div>
    <div>
      <label class="label" for="rv-reason">Reason</label>
      <select id="rv-reason" class="input" bind:value={reasonCode}>
        <option value={1}>Key is superseded</option>
        <option value={2}>Key has been compromised</option>
        <option value={3}>Key is no longer used</option>
        <option value={0}>No reason specified</option>
      </select>
    </div>
    <div>
      <label class="label" for="rv-note">Note (optional)</label>
      <input id="rv-note" class="input" bind:value={reason} placeholder="Lost hardware token" />
    </div>
    <PassphraseInput bind:value={passphrase} label="Key passphrase" id="rv-pass" />
  </div>

  {#snippet footer()}
    <button class="btn-secondary" onclick={onClose} disabled={busy}>Cancel</button>
    <button class="btn-danger" onclick={revoke} disabled={busy || !passphrase}>
      {#if busy}<Spinner size={16} />{/if} Revoke &amp; download
    </button>
  {/snippet}
</Dialog>
