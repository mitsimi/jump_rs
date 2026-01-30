import { useMutation } from "@tanstack/react-query";
import { arpLookup } from "../api/generated";

export function useMacLookup() {
  return useMutation({
    mutationFn: async (ip: string) => {
      const { data } = await arpLookup({ body: { ip }, throwOnError: true });
      return data;
    },
  });
}
