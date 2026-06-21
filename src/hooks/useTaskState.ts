import { useAppStore } from "../store/appStore";

export function useTaskState() {
  const { busy, setBusy, job, setJob } = useAppStore();
  const processingTaskActive = busy !== "";

  return {
    busy,
    setBusy,
    job,
    setJob,
    processingTaskActive,
  };
}
