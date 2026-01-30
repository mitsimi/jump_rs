import { useMutation, useQueryClient } from "@tanstack/react-query";
import { importDevices } from "../api/generated";
import type { ImportRequest } from "../api/generated";

export function useImportDevices() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: ImportRequest[]) => {
      const { data: response } = await importDevices({
        body: data,
        throwOnError: true,
      });
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
