import { useCallback, useRef, useState } from "react";

export function useNotice() {
  const [notice, setNotice] = useState("");
  const timerRef = useRef<ReturnType<typeof setTimeout>>();

  const showNotice = useCallback((message: string, duration = 5000) => {
    setNotice(message);
    if (timerRef.current) {
      clearTimeout(timerRef.current);
    }
    if (duration > 0) {
      timerRef.current = setTimeout(() => {
        setNotice("");
      }, duration);
    }
  }, []);

  return { notice, setNotice, showNotice };
}
