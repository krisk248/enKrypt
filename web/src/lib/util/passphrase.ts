// A lightweight passphrase strength estimator (no external deps). Returns a
// 0–4 score plus a label and hint — enough for a useful meter, not a promise.

export interface Strength {
  score: 0 | 1 | 2 | 3 | 4;
  label: string;
  hint: string;
}

export function estimateStrength(pw: string): Strength {
  if (!pw) return { score: 0, label: 'Empty', hint: 'Enter a passphrase' };

  let bits = 0;
  const classes = [/[a-z]/, /[A-Z]/, /[0-9]/, /[^a-zA-Z0-9]/];
  const poolSize = classes.reduce((n, re) => n + (re.test(pw) ? 1 : 0), 0);
  const alphabet = [26, 52, 62, 95][Math.max(0, poolSize - 1)] ?? 26;
  bits = Math.log2(alphabet) * pw.length;

  // Penalise obvious repetition / sequences.
  if (/(.)\1{2,}/.test(pw)) bits -= 10;
  if (/^(?:password|qwerty|12345|letmein|admin)/i.test(pw)) bits -= 30;

  let score: Strength['score'];
  if (bits < 28) score = 0;
  else if (bits < 40) score = 1;
  else if (bits < 60) score = 2;
  else if (bits < 90) score = 3;
  else score = 4;

  const labels = ['Very weak', 'Weak', 'Fair', 'Strong', 'Very strong'];
  const hints = [
    'Use a longer, more varied passphrase',
    'Add length and mix character types',
    'A few more words would help',
    'Good — a solid passphrase',
    'Excellent passphrase',
  ];
  return { score, label: labels[score], hint: hints[score] };
}
