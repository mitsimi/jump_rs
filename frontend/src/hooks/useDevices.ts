import { useQuery } from "@tanstack/react-query";
import { getDevices } from "../api/generated";

export function useDevices() {
  return useQuery({
    queryKey: ["devices"],
    queryFn: async () => {
      const { data } = await getDevices({ throwOnError: true });
      return data;
    },
    staleTime: 30000,
    retry: 1,
  });
}
