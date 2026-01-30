import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDevice } from "../api/generated";
import type { Device, UpdateDeviceRequest } from "../api/generated";

export function useUpdateDevice() {
  const queryClient = useQueryClient();
  const queryKey = ["devices"];

  return useMutation({
    mutationFn: async ({
      id,
      data,
    }: {
      id: string;
      data: UpdateDeviceRequest;
    }) => {
      const { data: response } = await updateDevice({
        path: { id },
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
