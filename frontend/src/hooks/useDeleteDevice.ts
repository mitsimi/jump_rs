import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteDevice } from "../api/generated";
import type { Device } from "../api/generated";

export function useDeleteDevice() {
  const queryClient = useQueryClient();
  const queryKey = ["devices"];

  return useMutation({
    mutationFn: async (id: string) => {
      await deleteDevice({ path: { id }, throwOnError: true });
    },
    onMutate: async (id) => {
      await queryClient.cancelQueries({ queryKey });
      const previousDevices = queryClient.getQueryData<Device[]>(queryKey);

      queryClient.setQueryData<Device[]>(queryKey, (old) =>
        old ? old.filter((device) => device.id !== id) : [],
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
