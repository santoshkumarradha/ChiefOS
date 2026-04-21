import briefState from "../mock/brief-state.json";
import MorningBrief, { type BriefState } from "./MorningBrief";

export default function App() {
  return <MorningBrief brief={briefState as BriefState} />;
}
