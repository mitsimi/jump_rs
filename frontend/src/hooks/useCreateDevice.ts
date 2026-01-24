import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createDevice } from "../api/devices";
import type { Device, DeviceFormData } from "../types/device";

export function useCreateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (data: DeviceFormData) => createDevice(data),
    onMutate: async (_newDevice: DeviceFormData) => {
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: ["devices"] });

      // Snapshot the previous value
      const previousDevices = queryClient.getQueryData<Device[]>(["devices"]);

      // Return context with the snapshot
      return { previousDevices };
    },
    onSuccess: (newDevice) => {
      // Update cache with the new device from server
      queryClient.setQueryData<Device[]>(["devices"], (old) => 
        old ? [...old, newDevice] : [newDevice]
      );
    },
    onError: (_err, _newDevice, context) => {
      // Rollback on error
      if (context?.previousDevices) {
        queryClient.setQueryData(["devices"], context.previousDevices);
      }
    },
    onSettled: () => {
      // Refetch to ensure sync with server
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
