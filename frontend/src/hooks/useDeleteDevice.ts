import { useMutation, useQueryClient } from "@tanstack/react-query";
import { deleteDevice } from "../api/devices";

export function useDeleteDevice() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => deleteDevice(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["devices"] });
    },
  });
}
