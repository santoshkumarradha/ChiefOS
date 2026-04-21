import { useBriefState } from "./hooks/useBriefState";
import OfflineBanner from "./components/OfflineBanner";
import MorningBrief from "./MorningBrief";

const CHIEF_CORE_URL: string = (import.meta as any).env.VITE_CHIEF_CORE_URL || "http://127.0.0.1:4711";

export default function App() {
  const { brief, status } = useBriefState();

  return (
    <>
      {status === "offline" && <OfflineBanner url={CHIEF_CORE_URL} />}
      <MorningBrief brief={brief} isOffline={status === "offline"} />
    </>
  );
}
