import { useCallback } from "react";
import { useAuth } from "./useAuth";

export function useAuthErrorHandler() {
  const { logout } = useAuth();

  const handleAuthError = useCallback(
    (error: unknown) => {
      if (error && typeof error === "object" && "status" in error) {
        const errorWithStatus = error as { status?: number };
        if (errorWithStatus.status === 401) {
          logout();
          return true;
        }
      }
      return false;
    },
    [logout],
  );

  return { handleAuthError };
}
