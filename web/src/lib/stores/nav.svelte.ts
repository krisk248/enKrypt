// App navigation + cross-view intents (e.g. "sign a file with this key").

type View = 'keys' | 'encrypt' | 'decrypt' | 'sign' | 'text' | 'settings';

class NavStore {
  view = $state<View>('keys');
  /** Fingerprint of a key a user chose to sign/encrypt with from the Key Manager. */
  signerFpr = $state<string | null>(null);
  recipientFpr = $state<string | null>(null);

  go(view: View) {
    this.view = view;
  }

  signWith(fpr: string) {
    this.signerFpr = fpr;
    this.view = 'sign';
  }
}

export const nav = new NavStore();
