import { useMutation } from "@tanstack/react-query";
import { lookupMacAddress } from "../api/devices";

export function useMacLookup() {
  return useMutation({
    mutationFn: (ip: string) => lookupMacAddress(ip),
  });
}
