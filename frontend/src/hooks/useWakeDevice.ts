import { useMutation } from "@tanstack/react-query";
import { wakeDeviceMutation } from "../api/generated/@tanstack/react-query.gen";

export function useWakeDevice() {
  return useMutation(wakeDeviceMutation());
}
