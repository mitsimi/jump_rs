import { useMutation } from "@tanstack/react-query";
import { login as loginApi } from "../api/generated";
import { useAuth } from "./useAuth";

export function useLogin() {
  const { login: setLoggedIn } = useAuth();

  return useMutation({
    mutationFn: async ({
      username,
      password,
    }: {
      username: string;
      password: string;
    }) => {
      const { data, error, response } = await loginApi({
        body: { username, password },
      });

      if (response?.status === 401 || response?.status === 403 || error) {
        const errorResponse = error as { message?: string } | undefined;
        const fallbackMessage =
          response?.status === 403
            ? "Origin not allowed. Update server allow_origins to include this frontend."
            : "Invalid username or password";
        throw new Error(
          errorResponse?.message || fallbackMessage,
        );
      }

      if (!data) {
        throw new Error("Login failed");
      }

      return data;
    },
    onSuccess: () => {
      setLoggedIn();
    },
  });
}
