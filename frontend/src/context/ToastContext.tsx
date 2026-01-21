import {
  createContext,
  useContext,
  useState,
  type ReactNode,
  type Context,
} from "react";
import type { ToastType } from "../types/device";

interface ToastMessage {
  id: string;
  message: string;
  type: ToastType;
}

interface ToastContextType {
  showToast: (message: string, type: ToastType) => void;
}

export const ToastContext: Context<ToastContextType> =
  createContext<ToastContextType>({
    showToast: () => {},
  });

export function useToast() {
  return useContext(ToastContext);
}

export function createToastProvider(
  ToastComponent: React.ComponentType<{ toasts: ToastMessage[] }>,
) {
  return function ToastProvider({ children }: { children: ReactNode }) {
    const [toasts, setToasts] = useState<ToastMessage[]>([]);

    const showToast = (message: string, type: ToastType) => {
      const id = crypto.randomUUID();
      setToasts((prev: ToastMessage[]) => [...prev, { id, message, type }]);
      setTimeout(() => {
        setToasts((prev: ToastMessage[]) =>
          prev.filter((t: ToastMessage) => t.id !== id),
        );
      }, 5000);
    };

    return (
      <ToastContext.Provider value={{ showToast }}>
        {children}
        <ToastComponent toasts={toasts} />
      </ToastContext.Provider>
    );
  };
}
