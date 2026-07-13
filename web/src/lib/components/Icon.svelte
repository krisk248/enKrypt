<script lang="ts" module>
  // Minimal inline icon set (stroke-based, 24px grid). Bundled, no CDN.
  export const paths: Record<string, string> = {
    keys: 'M15 7a4 4 0 1 0-3.9 5H14l1.5 1.5L17 12l1.5 1.5L20 12l-1.5-1.5m-3.5 0a4 4 0 0 1-1-3',
    lock: 'M7 11V8a5 5 0 0 1 10 0v3M5 11h14v9a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1z',
    unlock: 'M7 11V8a5 5 0 0 1 9.9-1M5 11h14v9a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1z',
    sign: 'M4 20h16M4 16l9-9 3 3-9 9H4z M13 7l3 3',
    text: 'M4 6h16M4 12h16M4 18h10',
    settings:
      'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z M19 12a7 7 0 0 0-.1-1l2-1.6-2-3.5-2.4 1a7 7 0 0 0-1.7-1L14.5 2h-5l-.3 2.9a7 7 0 0 0-1.7 1l-2.4-1-2 3.5 2 1.6a7 7 0 0 0 0 2l-2 1.6 2 3.5 2.4-1a7 7 0 0 0 1.7 1l.3 2.9h5l.3-2.9a7 7 0 0 0 1.7-1l2.4 1 2-3.5-2-1.6c.06-.3.1-.66.1-1z',
    plus: 'M12 5v14M5 12h14',
    upload: 'M12 15V3m0 0-4 4m4-4 4 4M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2',
    download: 'M12 3v12m0 0 4-4m-4 4-4-4M4 17v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2',
    trash: 'M4 7h16M10 11v6M14 11v6M5 7l1 13a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1l1-13M9 7V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v3',
    copy: 'M9 9V5a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1h-4M4 10a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v9a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1z',
    clipboard: 'M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2M9 5a2 2 0 0 0 2 2h2a2 2 0 0 0 2-2M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2',
    check: 'M20 6 9 17l-5-5',
    x: 'M18 6 6 18M6 6l12 12',
    warn: 'M12 9v4m0 4h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z',
    shield: 'M12 3 4 6v6c0 5 3.4 8.5 8 9 4.6-.5 8-4 8-9V6l-8-3z',
    'shield-check': 'M12 3 4 6v6c0 5 3.4 8.5 8 9 4.6-.5 8-4 8-9V6l-8-3z M9 12l2 2 4-4',
    search: 'M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.3-4.3',
    sun: 'M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM12 1v2M12 21v2M4.2 4.2l1.4 1.4M18.4 18.4l1.4 1.4M1 12h2M21 12h2M4.2 19.8l1.4-1.4M18.4 5.6l1.4-1.4',
    moon: 'M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z',
    star: 'M12 3l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5L12 17.8 6.2 20.9l1.1-6.5L2.6 9.8l6.5-.9L12 3z',
    eye: 'M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7z M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z',
    'eye-off': 'M3 3l18 18M10.6 10.6a3 3 0 0 0 4.2 4.2M9.4 5.2A9.5 9.5 0 0 1 12 5c6 0 10 7 10 7a17 17 0 0 1-2.8 3.4M6.1 6.1A17 17 0 0 0 2 12s4 7 10 7a9.5 9.5 0 0 0 2.6-.4',
    info: 'M12 16v-4m0-4h.01M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18z',
    file: 'M14 3v4a1 1 0 0 0 1 1h4M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8l-5-5z',
    dots: 'M12 6h.01M12 12h.01M12 18h.01',
    revoke: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18zM5.6 5.6l12.8 12.8',
  };
</script>

<script lang="ts">
  interface Props {
    name: string;
    size?: number;
    class?: string;
    fill?: boolean;
  }
  let { name, size = 20, class: klass = '', fill = false }: Props = $props();
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={fill ? 'currentColor' : 'none'}
  stroke={fill ? 'none' : 'currentColor'}
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={klass}
  aria-hidden="true"
>
  <path d={paths[name] ?? ''} />
</svg>
