// Theme store — light / dark / system, persisted to localStorage. The initial
// class is applied pre-paint by /public/theme-init.js; this keeps it in sync.

type Mode = 'light' | 'dark' | 'system';

const KEY = 'enkrypt.theme';

function systemPrefersDark() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches;
}

class ThemeStore {
  mode = $state<Mode>('system');

  constructor() {
    const saved = localStorage.getItem(KEY);
    this.mode = saved === 'dark' || saved === 'light' ? saved : 'system';
    // React to OS changes when in system mode.
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (this.mode === 'system') this.apply();
    });
    this.apply();
  }

  get isDark(): boolean {
    return this.mode === 'dark' || (this.mode === 'system' && systemPrefersDark());
  }

  private apply() {
    document.documentElement.classList.toggle('dark', this.isDark);
  }

  set(mode: Mode) {
    this.mode = mode;
    if (mode === 'system') localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, mode);
    this.apply();
  }

  toggle() {
    this.set(this.isDark ? 'light' : 'dark');
  }
}

export const theme = new ThemeStore();
