import { useMutation, useQueryClient } from "@tanstack/react-query";
import { createDevice } from "../api/generated";
import type { Device, CreateDeviceRequest } from "../api/generated";

export function useCreateDevice() {
  const queryClient = useQueryClient();
  const queryKey = ["devices"];

  return useMutation({
    mutationFn: async (data: CreateDeviceRequest) => {
      const { data: response } = await createDevice({
        body: data,
        throwOnError: true,
      });
      return response as Device;
    },
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
