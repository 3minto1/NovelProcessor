import { useCallback, useEffect, useState } from "react";

export function useNotice() {
  const [notice, setNotice] = useState("");
  const [duration, setDuration] = useState(5000);

  useEffect(() => {
    if (!notice) return undefined;
    const timer = window.setTimeout(() => setNotice(""), duration);
    return () => window.clearTimeout(timer);
  }, [duration, notice]);

  const showNotice = useCallback(
    (message: string, nextDuration = 5000) => {
      setDuration(nextDuration);
      setNotice(message);
    },
    []
  );

  return { notice, setNotice, showNotice };
}
