import { useQuery } from "@tanstack/react-query";
import { exportDevices } from "../api/generated";

export function useExportDevices() {
  return useQuery({
    queryKey: ["devices", "export"],
    queryFn: async () => {
      const { data } = await exportDevices({ throwOnError: true });
      return data;
    },
    enabled: false,
  });
}
