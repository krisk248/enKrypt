<script lang="ts">
  import Icon from './Icon.svelte';
  import { estimateStrength } from '../util/passphrase';

  interface Props {
    value: string;
    placeholder?: string;
    label?: string;
    meter?: boolean;
    autofocus?: boolean;
    id?: string;
  }
  let {
    value = $bindable(),
    placeholder = 'Passphrase',
    label = '',
    meter = false,
    autofocus = false,
    id = 'passphrase',
  }: Props = $props();

  let show = $state(false);
  let strength = $derived(meter ? estimateStrength(value) : null);
  const barColors = ['bg-red-500', 'bg-orange-500', 'bg-yellow-500', 'bg-green-500', 'bg-green-600'];
</script>

<div>
  {#if label}<label class="label" for={id}>{label}</label>{/if}
  <div class="relative">
    <!-- svelte-ignore a11y_autofocus -->
    <input
      {id}
      type={show ? 'text' : 'password'}
      class="input pr-10"
      {placeholder}
      autocomplete="off"
      autocapitalize="off"
      spellcheck="false"
      autofocus={autofocus || undefined}
      bind:value
    />
    <button
      type="button"
      class="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
      onclick={() => (show = !show)}
      aria-label={show ? 'Hide passphrase' : 'Show passphrase'}
      tabindex="-1"
    >
      <Icon name={show ? 'eye-off' : 'eye'} size={18} />
    </button>
  </div>
  {#if meter && strength}
    <div class="mt-1.5 flex items-center gap-2">
      <div class="flex h-1.5 flex-1 gap-1">
        {#each [0, 1, 2, 3, 4] as i}
          <div
            class="h-full flex-1 rounded-full transition-colors {i <= strength.score
              ? barColors[strength.score]
              : 'bg-slate-200 dark:bg-slate-700'}"
          ></div>
        {/each}
      </div>
      <span class="w-24 text-right text-xs text-slate-500 dark:text-slate-400">{strength.label}</span>
    </div>
  {/if}
</div>
