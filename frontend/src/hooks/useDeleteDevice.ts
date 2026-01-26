import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteDevice } from "../api/devices";
import type { Device } from "../types/device";

export function useDeleteDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteDevice(id),
    onMutate: async (id: string) => {
      await queryClient.cancelQueries({ queryKey: ["devices"] });

      const previousDevices = queryClient.getQueryData<Device[]>(["devices"]);

      queryClient.setQueryData<Device[]>(["devices"], (old) =>
        old ? old.filter((device) => device.id !== id) : [],
      );

      return { previousDevices };
    },
    onError: (_err, _id, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(["devices"], context.previousDevices);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
