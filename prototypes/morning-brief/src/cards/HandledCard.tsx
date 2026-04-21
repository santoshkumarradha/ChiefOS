import { motion, type Variants } from "framer-motion";
import type { HandledItem } from "../MorningBrief";

type HandledCardProps = {
  item: HandledItem;
  variants: Variants;
  actionRef: (node: HTMLButtonElement | null) => void;
  onOpen: () => void;
};

export default function HandledCard({
  item,
  variants,
  actionRef,
  onOpen,
}: HandledCardProps) {
  return (
    <motion.article className="handled-card-shell" variants={variants}>
      <button
        ref={actionRef}
        className="handled-card"
        type="button"
        onClick={onOpen}
        aria-label={`Open receipt for ${item.title}`}
      >
        <span className="handled-category">{item.category}</span>
        <strong>{item.title}</strong>
        <span>{item.outcome}</span>
        <span className="handled-footer">
          <span>{item.agent}</span>
          <span>{item.confidence}%</span>
        </span>
      </button>
    </motion.article>
  );
}
