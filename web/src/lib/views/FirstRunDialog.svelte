<script lang="ts">
  import Icon from '../components/Icon.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import Spinner from '../components/Spinner.svelte';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';

  let choice = $state<'session' | 'persistent' | null>(null);
  let useVault = $state(true);
  let vaultPass = $state('');
  let vaultPass2 = $state('');
  let busy = $state(false);

  async function confirm() {
    if (!choice) return;
    if (choice === 'persistent' && useVault) {
      if (vaultPass.length < 8) return toasts.error('Vault passphrase must be at least 8 characters.');
      if (vaultPass !== vaultPass2) return toasts.error('Vault passphrases do not match.');
    }
    busy = true;
    try {
      await keyring.chooseStorage(
        choice,
        choice === 'persistent' && useVault ? vaultPass : undefined,
      );
    } catch (e) {
      toasts.error((e as Error).message);
    } finally {
      busy = false;
    }
  }
</script>

<div class="flex h-full items-center justify-center bg-gradient-to-b from-slate-50 to-slate-100 p-6 dark:from-slate-950 dark:to-slate-900">
  <div class="card w-full max-w-xl p-7">
    <div class="mb-5 flex items-center gap-3">
      <div class="grid h-11 w-11 place-items-center rounded-xl bg-gradient-to-b from-brand-400 to-brand-600 text-white">
        <Icon name="shield" size={24} fill />
      </div>
      <div>
        <h1 class="text-lg font-bold">Welcome to enKrypt</h1>
        <p class="text-sm text-slate-500 dark:text-slate-400">Choose how your keys are stored. You can change this later.</p>
      </div>
    </div>

    <div class="grid gap-3 sm:grid-cols-2">
      <button
        class="rounded-xl border-2 p-4 text-left transition-colors
          {choice === 'session' ? 'border-brand-500 bg-brand-50 dark:bg-brand-950/40' : 'border-slate-200 hover:border-slate-300 dark:border-slate-700'}"
        onclick={() => (choice = 'session')}
      >
        <Icon name="unlock" size={22} class="mb-2 text-brand-600 dark:text-brand-300" />
        <h3 class="font-semibold">Session only</h3>
        <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">
          Keys live in memory and vanish when you close the tab. Nothing is written to disk. Export before leaving.
        </p>
      </button>

      <button
        class="rounded-xl border-2 p-4 text-left transition-colors
          {choice === 'persistent' ? 'border-brand-500 bg-brand-50 dark:bg-brand-950/40' : 'border-slate-200 hover:border-slate-300 dark:border-slate-700'}"
        onclick={() => (choice = 'persistent')}
      >
        <Icon name="lock" size={22} class="mb-2 text-brand-600 dark:text-brand-300" />
        <h3 class="font-semibold">Persistent</h3>
        <p class="mt-1 text-xs text-slate-500 dark:text-slate-400">
          Keyring stored locally in IndexedDB. Secret keys stay OpenPGP passphrase-encrypted.
        </p>
      </button>
    </div>

    {#if choice === 'persistent'}
      <div class="mt-4 rounded-xl border border-slate-200 p-4 dark:border-slate-700">
        <label class="flex items-center gap-2.5 text-sm font-medium">
          <input type="checkbox" bind:checked={useVault} class="h-4 w-4 rounded accent-brand-600" />
          Add a vault passphrase (extra AES-256-GCM + Argon2id layer)
        </label>
        {#if useVault}
          <div class="mt-3 grid gap-3 sm:grid-cols-2">
            <PassphraseInput bind:value={vaultPass} placeholder="Vault passphrase" meter id="vp1" />
            <PassphraseInput bind:value={vaultPass2} placeholder="Confirm passphrase" id="vp2" />
          </div>
          <p class="mt-2 text-xs text-slate-500 dark:text-slate-400">
            You'll be asked for this each time you open the app. It cannot be recovered if forgotten.
          </p>
        {/if}
      </div>
    {/if}

    <div class="mt-6 flex justify-end">
      <button class="btn-primary" disabled={!choice || busy} onclick={confirm}>
        {#if busy}<Spinner size={16} />{/if}
        Continue
      </button>
    </div>
  </div>
</div>
