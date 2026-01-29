import { useMutation } from "@tanstack/react-query";
import { arpLookupMutation } from "../api/generated/@tanstack/react-query.gen";

export function useMacLookup() {
  return useMutation(arpLookupMutation());
}
