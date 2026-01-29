import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  deleteDeviceMutation,
  getDevicesQueryKey,
} from "../api/generated/@tanstack/react-query.gen";
import type { Device } from "../api/generated";

export function useDeleteDevice() {
  const queryClient = useQueryClient();
  const queryKey = getDevicesQueryKey();

  return useMutation({
    ...deleteDeviceMutation(),
    onMutate: async (options) => {
      await queryClient.cancelQueries({ queryKey });
      const previousDevices = queryClient.getQueryData<Device[]>(queryKey);

      queryClient.setQueryData<Device[]>(queryKey, (old) =>
        old ? old.filter((device) => device.id !== options.path.id) : [],
      );

      return { previousDevices };
    },
    onError: (_err, _id, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKey, context.previousDevices);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey });
    },
  });
}
