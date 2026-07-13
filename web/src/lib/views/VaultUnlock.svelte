<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import Spinner from '../components/Spinner.svelte';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';

  let pass = $state('');
  let busy = $state(false);

  async function unlock(e: Event) {
    e.preventDefault();
    if (!pass) return;
    busy = true;
    try {
      await keyring.unlockVault(pass);
      pass = '';
      toasts.success('Vault unlocked.');
    } catch (err) {
      toasts.error('Wrong vault passphrase.');
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex h-full items-center justify-center bg-gradient-to-b from-slate-50 to-slate-100 p-6 dark:from-slate-950 dark:to-slate-900">
  <form class="card w-full max-w-sm p-7 text-center" onsubmit={unlock}>
    <div class="mx-auto mb-4 grid h-14 w-14 place-items-center rounded-2xl bg-gradient-to-b from-brand-400 to-brand-600 text-white">
      <Icon name="lock" size={26} fill />
    </div>
    <h1 class="text-lg font-bold">Unlock your vault</h1>
    <p class="mb-5 mt-1 text-sm text-slate-500 dark:text-slate-400">
      Enter your vault passphrase to decrypt your keyring.
    </p>
    <PassphraseInput bind:value={pass} placeholder="Vault passphrase" autofocus id="unlock" />
    <button class="btn-primary mt-4 w-full" disabled={busy || !pass}>
      {#if busy}<Spinner size={16} />{/if}
      Unlock
    </button>
  </form>
</div>
