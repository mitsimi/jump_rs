import { useQuery } from "@tanstack/react-query";
import { fetchDevices } from "../api/devices";

export function useDevices() {
  return useQuery({
    queryKey: ["devices"],
    queryFn: fetchDevices,
    staleTime: 30000,
    retry: 1,
  });
}
