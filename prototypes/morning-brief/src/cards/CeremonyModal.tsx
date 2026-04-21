import { useEffect, useRef } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { NeedItem } from "../MorningBrief";

type CeremonyModalProps = {
  item: NeedItem | null;
  onClose: () => void;
};

export default function CeremonyModal({ item, onClose }: CeremonyModalProps) {
  const closeRef = useRef<HTMLButtonElement | null>(null);

  useEffect(() => {
    if (!item) {
      return;
    }

    closeRef.current?.focus();
  }, [item]);

  return (
    <AnimatePresence>
      {item ? (
        <motion.div
          className="modal-backdrop"
          initial={{ opacity: 0 }}
          animate={{
            opacity: 1,
            transition: { type: "spring", stiffness: 260, damping: 28 },
          }}
          exit={{
            opacity: 0,
            transition: { type: "spring", stiffness: 320, damping: 32 },
          }}
          onMouseDown={(event) => {
            if (event.currentTarget === event.target) {
              onClose();
            }
          }}
        >
          <motion.section
            className="ceremony-modal"
            role="dialog"
            aria-modal="true"
            aria-labelledby="evidence-title"
            initial={{ opacity: 0, y: 24, scale: 0.97 }}
            animate={{
              opacity: 1,
              y: 0,
              scale: 1,
              transition: { type: "spring", stiffness: 320, damping: 30 },
            }}
            exit={{
              opacity: 0,
              y: 16,
              scale: 0.98,
              transition: { type: "spring", stiffness: 360, damping: 34 },
            }}
          >
            <div className="receipt-header">
              <div>
                <p className="section-kicker">Evidence card</p>
                <h2 id="evidence-title">{item.title}</h2>
              </div>
              <button
                ref={closeRef}
                className="icon-button"
                type="button"
                aria-label="Close evidence card"
                onClick={onClose}
              >
                x
              </button>
            </div>
            <p className="modal-summary">{item.summary}</p>
            <dl className="receipt-facts">
              <div>
                <dt>Agent</dt>
                <dd>{item.agent}</dd>
              </div>
              <div>
                <dt>Deadline</dt>
                <dd>{item.deadline}</dd>
              </div>
              <div>
                <dt>Trust delta</dt>
                <dd>{item.trust_delta > 0 ? "+" : ""}{item.trust_delta}</dd>
              </div>
            </dl>
            <ol className="provenance-chain">
              {item.evidence.map((entry) => (
                <li key={entry}>{entry}</li>
              ))}
              <li>Awaiting human approval</li>
            </ol>
          </motion.section>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
