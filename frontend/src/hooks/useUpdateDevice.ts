import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDevice } from "../api/devices";
import type { DeviceFormData } from "../types/device";

export function useUpdateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: DeviceFormData }) =>
      updateDevice(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
