import styles from "./Toast.module.css";
import { createToastProvider } from "../context/ToastContext";
import type { ToastMessage } from "../types/device";

interface ToastQueueProps {
  toasts: ToastMessage[];
}

export function ToastQueue({ toasts }: ToastQueueProps) {
  return (
    <div className={styles.container}>
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`${styles.toast} ${styles[toast.type]} ${styles.show}`}
        >
          <span className={styles.icon}>
            {toast.type === "success" ? "✓" : "✕"}
          </span>
          <span className={styles.message}>{toast.message}</span>
        </div>
      ))}
    </div>
  );
}

export const ToastProvider = createToastProvider(ToastQueue);

export function Toast() {
  return null;
}
