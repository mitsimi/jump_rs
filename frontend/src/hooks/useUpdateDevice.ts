import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  updateDeviceMutation,
  getDevicesQueryKey,
} from "../api/generated/@tanstack/react-query.gen";
import type { Device } from "../api/generated";

export function useUpdateDevice() {
  const queryClient = useQueryClient();
  const queryKey = getDevicesQueryKey();

  return useMutation({
    ...updateDeviceMutation(),
    onMutate: async () => {
      await queryClient.cancelQueries({ queryKey });
      const previousDevices = queryClient.getQueryData<Device[]>(queryKey);
      return { previousDevices };
    },
    onSuccess: (updatedDevice) => {
      queryClient.setQueryData<Device[]>(queryKey, (old) =>
        old
          ? old.map((device) =>
              device.id === updatedDevice.id ? updatedDevice : device,
            )
          : [updatedDevice],
      );
    },
    onError: (_err, _variables, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKey, context.previousDevices);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey });
    },
  });
}
