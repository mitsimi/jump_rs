import { useMutation, useQueryClient } from "@tanstack/react-query";
import { importDevices } from "../api/devices";
import type { ExportDevice } from "../types/device";

export function useImportDevices() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (devices: ExportDevice[]) => importDevices(devices),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
