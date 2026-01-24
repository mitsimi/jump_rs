import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteDevice } from "../api/devices";
import type { Device } from "../types/device";

export function useDeleteDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteDevice(id),
    onMutate: async (id: string) => {
      // Cancel any outgoing refetches
      await queryClient.cancelQueries({ queryKey: ["devices"] });

      // Snapshot the previous value
      const previousDevices = queryClient.getQueryData<Device[]>(["devices"]);

      // Optimistically update by removing the device
      queryClient.setQueryData<Device[]>(["devices"], (old) => 
        old ? old.filter(device => device.id !== id) : []
      );

      // Return context with the snapshot
      return { previousDevices };
    },
    onError: (_err, _id, context) => {
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
