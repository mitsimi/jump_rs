import { useQuery } from "@tanstack/react-query";
import { getDevicesOptions } from "../api/generated/@tanstack/react-query.gen";

export function useDevices() {
  return useQuery({
    ...getDevicesOptions(),
    staleTime: 30000,
    retry: 1,
  });
}
