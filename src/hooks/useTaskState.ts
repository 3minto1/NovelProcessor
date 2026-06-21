import { useAppStore } from "../store/appStore";

export function useTaskState() {
  const busy = useAppStore((state) => state.busy);
  const setBusy = useAppStore((state) => state.setBusy);
  const job = useAppStore((state) => state.job);
  const setJob = useAppStore((state) => state.setJob);
  const processingTaskActive = busy !== "";

  return {
    busy,
    setBusy,
    job,
    setJob,
    processingTaskActive
  };
}
