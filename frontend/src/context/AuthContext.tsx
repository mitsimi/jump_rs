import { useCallback, type ReactNode, useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { authStatus, me } from "../api/generated";
import { AuthContext, type AuthContextType } from "./authTypes";

export { AuthContext, type AuthContextType } from "./authTypes";

export function AuthProvider({ children }: { children: ReactNode }) {
  const { data, isLoading, refetch } = useQuery({
    queryKey: ["auth", "status"],
    queryFn: async () => {
      const [statusResult, meResult] = await Promise.all([authStatus(), me()]);

      const authRequired = statusResult.data?.auth_required ?? true;

      if (!authRequired) {
        return {
          isAuthenticated: true,
          isAuthRequired: false,
          username: null,
        };
      }

      if (meResult.response?.status === 401) {
        return {
          isAuthenticated: false,
          isAuthRequired: true,
          username: null,
        };
      }

      if (meResult.data) {
        return {
          isAuthenticated: true,
          isAuthRequired: true,
          username: meResult.data.username,
        };
      }

      return { isAuthenticated: false, isAuthRequired: true, username: null };
    },
    staleTime: 5 * 60 * 1000,
    retry: false,
  });

  const login = useCallback(() => {
    void refetch();
  }, [refetch]);

  const logout = useCallback(() => {
    void refetch();
  }, [refetch]);

  const checkAuth = useCallback(async () => {
    await refetch();
  }, [refetch]);

  const value: AuthContextType = useMemo(
    () => ({
      isAuthenticated: data?.isAuthenticated ?? false,
      isLoading,
      isAuthRequired: data?.isAuthRequired ?? true,
      username: data?.username ?? null,
      login,
      logout,
      checkAuth,
    }),
    [data, isLoading, login, logout, checkAuth],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
