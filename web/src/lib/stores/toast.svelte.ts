// Toast notifications.

export type ToastKind = 'success' | 'error' | 'info';

export interface Toast {
  id: number;
  kind: ToastKind;
  message: string;
}

class ToastStore {
  items = $state<Toast[]>([]);
  private seq = 0;

  push(kind: ToastKind, message: string, ttl = 4000) {
    const id = ++this.seq;
    this.items.push({ id, kind, message });
    if (ttl > 0) setTimeout(() => this.dismiss(id), ttl);
    return id;
  }

  success(message: string) {
    return this.push('success', message);
  }
  error(message: string) {
    return this.push('error', message, 6000);
  }
  info(message: string) {
    return this.push('info', message);
  }

  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new ToastStore();
