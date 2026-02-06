import { useMutation, useQueryClient } from "@tanstack/react-query";
import { logout as logoutApi } from "../api/generated";
import { useAuth } from "./useAuth";
import { useToast } from "./useToast";

export function useLogout() {
  const { logout: setLoggedOut } = useAuth();
  const queryClient = useQueryClient();
  const { showToast } = useToast();

  return useMutation({
    mutationFn: async () => {
      await logoutApi({ throwOnError: true });
    },
    onSuccess: () => {
      setLoggedOut();
      queryClient.removeQueries({ queryKey: ["devices"] });
      queryClient.removeQueries({ queryKey: ["devices", "export"] });
      queryClient.invalidateQueries({ queryKey: ["auth", "status"] });
    },
    onError: (error) => {
      const message =
        error instanceof Error ? error.message : "Logout failed";
      showToast(message, "error");
    },
  });
}
