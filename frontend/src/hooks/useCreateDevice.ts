import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  createDeviceMutation,
  getDevicesQueryKey,
} from "../api/generated/@tanstack/react-query.gen";
import type { Device } from "../api/generated";

export function useCreateDevice() {
  const queryClient = useQueryClient();
  const queryKey = getDevicesQueryKey();

  return useMutation({
    ...createDeviceMutation(),
    onMutate: async () => {
      await queryClient.cancelQueries({ queryKey });
      const previousDevices = queryClient.getQueryData<Device[]>(queryKey);
      return { previousDevices };
    },
    onSuccess: (newDevice) => {
      queryClient.setQueryData<Device[]>(queryKey, (old) =>
        old ? [...old, newDevice] : [newDevice],
      );
    },
    onError: (_err, _newDevice, context) => {
      if (context?.previousDevices) {
        queryClient.setQueryData(queryKey, context.previousDevices);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey });
    },
  });
}
