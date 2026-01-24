import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDevice } from "../api/devices";
import type { Device, DeviceFormData } from "../types/device";

export function useUpdateDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: DeviceFormData }) =>
      updateDevice(id, data),
    onMutate: async () => {
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: ["devices"] });

      // Snapshot the previous value
      const previousDevices = queryClient.getQueryData<Device[]>(["devices"]);

      // Return context with the snapshot
      return { previousDevices };
    },
    onSuccess: (updatedDevice) => {
      // Update cache with the updated device from server
      queryClient.setQueryData<Device[]>(["devices"], (old) => 
        old ? old.map(device => device.id === updatedDevice.id ? updatedDevice : device) : [updatedDevice]
      );
    },
    onError: (_err, _variables, context) => {
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
