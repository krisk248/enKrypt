<script lang="ts">
  import Dialog from '../components/Dialog.svelte';
  import PassphraseInput from '../components/PassphraseInput.svelte';
  import Spinner from '../components/Spinner.svelte';
  import { cryptoApi, CryptoError } from '../worker/rpc';
  import { keyring } from '../stores/keyring.svelte';
  import { toasts } from '../stores/toast.svelte';
  import type { KeyAlgo } from '../../wasm/enkrypt_core';

  interface Props {
    open: boolean;
    onClose: () => void;
  }
  let { open, onClose }: Props = $props();

  let name = $state('');
  let email = $state('');
  let comment = $state('');
  let passphrase = $state('');
  let algo = $state<KeyAlgo>('ed25519');
  let expiry = $state<'never' | '1y' | '2y' | '5y'>('never');
  let busy = $state(false);

  const algos: { id: KeyAlgo; label: string; desc: string }[] = [
    { id: 'ed25519', label: 'Ed25519 / Cv25519', desc: 'Modern · fast · recommended' },
    { id: 'rsa2048', label: 'RSA 2048', desc: 'Legacy · widest compatibility' },
    { id: 'rsa3072', label: 'RSA 3072', desc: 'Strong · broad compatibility' },
    { id: 'rsa4096', label: 'RSA 4096', desc: 'Maximum RSA strength' },
    { id: 'nist_p256', label: 'NIST P-256', desc: 'ECDSA/ECDH · enterprise' },
    { id: 'nist_p384', label: 'NIST P-384', desc: 'ECDSA/ECDH · high security' },
    { id: 'ed448', label: 'Ed448 / X448', desc: 'Modern · v6 · high security' },
    { id: 'post_quantum', label: 'Post-Quantum', desc: 'ML-KEM + ML-DSA · experimental' },
  ];

  function reset() {
    name = email = comment = passphrase = '';
    algo = 'ed25519';
    expiry = 'never';
  }

  function expirySecs(): number | undefined {
    const y = 365 * 24 * 3600;
    return { never: undefined, '1y': y, '2y': 2 * y, '5y': 5 * y }[expiry];
  }

  async function generate() {
    if (!name.trim()) return toasts.error('Please enter a name.');
    if (!email.trim()) return toasts.error('Please enter an email.');
    if (passphrase.length < 6) return toasts.error('Passphrase must be at least 6 characters.');
    busy = true;
    try {
      const bundle = await cryptoApi.generateKey({
        name: name.trim(),
        email: email.trim(),
        comment: comment.trim() || undefined,
        passphrase,
        algo,
        expiry_secs: expirySecs(),
      });
      await keyring.addGenerated(bundle);
      toasts.success(`Generated key for ${email.trim()}.`);
      reset();
      onClose();
    } catch (e) {
      const msg = e instanceof CryptoError ? e.message : (e as Error).message;
      toasts.error(`Key generation failed: ${msg}`);
    } finally {
      busy = false;
    }
  }
</script>

<Dialog {open} {onClose} title="Generate a new key" size="lg">
  <div class="space-y-4">
    <div class="grid gap-3 sm:grid-cols-2">
      <div>
        <label class="label" for="kg-name">Name</label>
        <input id="kg-name" class="input" bind:value={name} placeholder="Ada Lovelace" />
      </div>
      <div>
        <label class="label" for="kg-email">Email</label>
        <input id="kg-email" class="input" type="email" bind:value={email} placeholder="ada@example.com" />
      </div>
    </div>
    <div>
      <label class="label" for="kg-comment">Comment (optional)</label>
      <input id="kg-comment" class="input" bind:value={comment} placeholder="work key" />
    </div>

    <div>
      <span class="label">Algorithm</span>
      <div class="grid gap-2 sm:grid-cols-3">
        {#each algos as a}
          <button
            type="button"
            class="rounded-lg border-2 p-2.5 text-left transition-colors
              {algo === a.id ? 'border-brand-500 bg-brand-50 dark:bg-brand-950/40' : 'border-slate-200 hover:border-slate-300 dark:border-slate-700'}"
            onclick={() => (algo = a.id)}
          >
            <p class="text-sm font-semibold">{a.label}</p>
            <p class="text-xs text-slate-500 dark:text-slate-400">{a.desc}</p>
          </button>
        {/each}
      </div>
    </div>

    <div class="grid gap-3 sm:grid-cols-2">
      <PassphraseInput bind:value={passphrase} label="Passphrase" meter id="kg-pass" />
      <div>
        <label class="label" for="kg-expiry">Expiry</label>
        <select id="kg-expiry" class="input" bind:value={expiry}>
          <option value="never">Never</option>
          <option value="1y">1 year</option>
          <option value="2y">2 years</option>
          <option value="5y">5 years</option>
        </select>
      </div>
    </div>
    <p class="text-xs text-slate-400">
      The secret key is always stored passphrase-encrypted (OpenPGP S2K, AES-256).
    </p>
  </div>

  {#snippet footer()}
    <button class="btn-secondary" onclick={onClose} disabled={busy}>Cancel</button>
    <button class="btn-primary" onclick={generate} disabled={busy}>
      {#if busy}<Spinner size={16} />Generating…{:else}Generate key{/if}
    </button>
  {/snippet}
</Dialog>
