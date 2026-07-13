<script lang="ts">
  import Icon from './Icon.svelte';
  import { keyring } from '../stores/keyring.svelte';
  import { parseUserId } from '../util/format';

  let secretCount = $derived(keyring.secretKeys.length);
  let defaultUid = $derived(
    keyring.defaultKey ? parseUserId(keyring.defaultKey.info.primary_user_id ?? '').name : null,
  );
</script>

<footer class="flex items-center justify-between border-t border-slate-200 bg-white/70 px-4 py-1.5 text-xs text-slate-500 dark:border-slate-800 dark:bg-slate-900/50 dark:text-slate-400">
  <div class="flex items-center gap-4">
    <span>
      {keyring.keys.length} key{keyring.keys.length === 1 ? '' : 's'}
      ({secretCount} secret)
    </span>
    {#if defaultUid}
      <span class="flex items-center gap-1.5">
        <Icon name="star" size={13} fill class="text-amber-500" />
        Default: <strong class="font-medium text-slate-600 dark:text-slate-300">{defaultUid}</strong>
      </span>
    {/if}
  </div>
  <div class="flex items-center gap-1.5">
    <Icon name="shield-check" size={13} class="text-green-600" />
    <span>Offline · client-side only</span>
  </div>
</footer>
