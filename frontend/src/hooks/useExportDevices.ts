import { useQuery } from "@tanstack/react-query";
import { exportDevicesOptions } from "../api/generated/@tanstack/react-query.gen";

export function useExportDevices() {
  return useQuery({
    ...exportDevicesOptions(),
    enabled: false,
  });
}
