import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createDevice } from "../api/devices";
import type { Device, DeviceFormData } from "../types/device";

export function useCreateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: DeviceFormData) => createDevice(data),
    onMutate: async () => {
      await queryClient.cancelQueries({ queryKey: ["devices"] });

      const previousDevices = queryClient.getQueryData<Device[]>(["devices"]);

      return { previousDevices };
    },
    onSuccess: (newDevice) => {
      queryClient.setQueryData<Device[]>(["devices"], (old) =>
        old ? [...old, newDevice] : [newDevice],
      );
    },
    onError: (_err, _newDevice, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(["devices"], context.previousDevices);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
