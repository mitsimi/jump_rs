import { useMutation } from "@tanstack/react-query";
import { wakeDevice } from "../api/devices";

export function useWakeDevice() {
  return useMutation({
    mutationFn: (id: string) => wakeDevice(id),
  });
}
