import { useState, useCallback } from "react";

export function usePacketCount() {
  const [count, setCount] = useState(0);

  const increment = useCallback(() => {
    setCount((prev) => prev + 1);
  }, []);

  const reset = useCallback(() => {
    setCount(0);
  }, []);

  return {
    count,
    increment,
    reset,
  };
}
