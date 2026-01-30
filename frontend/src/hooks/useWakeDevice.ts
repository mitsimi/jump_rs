import { useMutation } from "@tanstack/react-query";
import { wakeDevice } from "../api/generated";

export function useWakeDevice() {
  return useMutation({
    mutationFn: async (id: string) => {
      await wakeDevice({ path: { id }, throwOnError: true });
    },
  });
}
