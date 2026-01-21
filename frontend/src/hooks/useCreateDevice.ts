import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createDevice } from "../api/devices";
import type { DeviceFormData } from "../types/device";

export function useCreateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: DeviceFormData) => createDevice(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
