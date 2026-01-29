import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  importDevicesMutation,
  getDevicesQueryKey,
} from "../api/generated/@tanstack/react-query.gen";

export function useImportDevices() {
  const queryClient = useQueryClient();
  const queryKey = getDevicesQueryKey();

  return useMutation({
    ...importDevicesMutation(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey });
    },
  });
}
