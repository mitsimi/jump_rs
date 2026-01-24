import { useMutation } from "@tanstack/react-query";
import { exportDevices } from "../api/devices";

export function useExportDevices() {
  return useMutation({
    mutationFn: () => exportDevices(),
  });
}
