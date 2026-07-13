<script lang="ts">
  interface Props {
    size?: number;
    class?: string;
  }
  let { size = 18, class: klass = '' }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  class="animate-spin {klass}"
  aria-hidden="true"
>
  <circle cx="12" cy="12" r="9" fill="none" stroke="currentColor" stroke-width="3" opacity="0.2" />
  <path d="M21 12a9 9 0 0 0-9-9" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
</svg>
