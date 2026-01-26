import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDevice } from "../api/devices";
import type { Device, DeviceFormData } from "../types/device";

export function useUpdateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: DeviceFormData }) =>
      updateDevice(id, data),
    onMutate: async () => {
      await queryClient.cancelQueries({ queryKey: ["devices"] });

      const previousDevices = queryClient.getQueryData<Device[]>(["devices"]);

      return { previousDevices };
    },
    onSuccess: (updatedDevice) => {
      queryClient.setQueryData<Device[]>(["devices"], (old) =>
        old
          ? old.map((device) =>
              device.id === updatedDevice.id ? updatedDevice : device,
            )
          : [updatedDevice],
      );
    },
    onError: (_err, _variables, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(["devices"], context.previousDevices);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
