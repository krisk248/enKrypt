<script lang="ts">
  import Icon from './Icon.svelte';
  import type { SignatureInfo } from '../../wasm/enkrypt_core';
  import { formatDate } from '../util/format';

  interface Props {
    sig: SignatureInfo;
  }
  let { sig }: Props = $props();

  const map = {
    valid: {
      cls: 'border-green-500/40 bg-green-50 text-green-800 dark:bg-green-950/50 dark:text-green-200',
      icon: 'shield-check',
      label: 'Valid signature',
    },
    invalid: {
      cls: 'border-red-500/40 bg-red-50 text-red-800 dark:bg-red-950/50 dark:text-red-200',
      icon: 'warn',
      label: 'INVALID signature',
    },
    unknown_key: {
      cls: 'border-amber-500/40 bg-amber-50 text-amber-800 dark:bg-amber-950/50 dark:text-amber-200',
      icon: 'info',
      label: 'Unknown signer',
    },
  } as const;

  let m = $derived(map[sig.status]);
</script>

<div class="flex items-start gap-3 rounded-lg border px-3.5 py-2.5 text-sm {m.cls}">
  <Icon name={m.icon} size={20} class="mt-0.5 shrink-0" />
  <div class="min-w-0">
    <p class="font-semibold">{m.label}</p>
    <div class="mt-0.5 space-y-0.5 text-xs opacity-90">
      {#if sig.signer_user_id}<p class="truncate">Signer: {sig.signer_user_id}</p>{/if}
      {#if sig.key_id}<p class="mono">Key ID: {sig.key_id}</p>{/if}
      {#if sig.created_at}<p>Signed: {formatDate(sig.created_at)}</p>{/if}
      {#if sig.status === 'unknown_key'}<p>No matching public key in your keyring.</p>{/if}
    </div>
  </div>
</div>
