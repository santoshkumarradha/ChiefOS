import { motion, type Variants } from "framer-motion";
import type { NeedItem } from "../MorningBrief";

type NeedsYouCardProps = {
  item: NeedItem;
  variants: Variants;
  actionRef: (node: HTMLButtonElement | null) => void;
  onReview: () => void;
  isOffline?: boolean;
};

export default function NeedsYouCard({
  item,
  variants,
  actionRef,
  onReview,
  isOffline = false,
}: NeedsYouCardProps) {
  return (
    <motion.article className="need-card" variants={variants}>
      <div className="card-topline">
        <span className={`priority-chip ${item.priority}`}>{item.priority}</span>
        <span>{item.category}</span>
      </div>
      <h3>{item.title}</h3>
      <p>{item.summary}</p>
      <div className="card-footer">
        <span>{item.agent}</span>
        <span>{item.estimated_minutes} min</span>
      </div>
      <button
        ref={actionRef}
        className="review-button"
        type="button"
        onClick={onReview}
        disabled={isOffline}
        title={isOffline ? "chief-core offline" : undefined}
      >
        Review
      </button>
    </motion.article>
  );
}
