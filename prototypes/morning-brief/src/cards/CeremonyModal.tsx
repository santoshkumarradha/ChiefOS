import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { postApprove } from "../api";
import type { NeedItem } from "../MorningBrief";

type CeremonyModalProps = {
  item: NeedItem | null;
  onClose: () => void;
  isOffline?: boolean;
};

export default function CeremonyModal({ item, onClose, isOffline = false }: CeremonyModalProps) {
  const closeRef = useRef<HTMLButtonElement | null>(null);
  const [isApproving, setIsApproving] = useState(false);
  const [approvalError, setApprovalError] = useState<string | null>(null);

  useEffect(() => {
    if (!item) {
      return;
    }

    closeRef.current?.focus();
  }, [item]);

  const handleApprove = async (ceremony: boolean) => {
    if (!item || isOffline) return;

    setIsApproving(true);
    setApprovalError(null);

    try {
      await postApprove(item.id, ceremony);
      onClose();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Approval failed";
      setApprovalError(message);
    } finally {
      setIsApproving(false);
    }
  };

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
                disabled={isApproving}
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
              <li>
                {isOffline
                  ? "chief-core offline - approval disabled"
                  : "Awaiting human approval"}
              </li>
            </ol>

            {approvalError && (
              <div
                style={{
                  color: "#d32f2f",
                  fontSize: "13px",
                  marginTop: "12px",
                  padding: "8px",
                  backgroundColor: "#ffebee",
                  borderRadius: "4px",
                }}
              >
                {approvalError}
              </div>
            )}

            {!isOffline && (
              <div
                style={{
                  display: "flex",
                  gap: "8px",
                  marginTop: "16px",
                  justifyContent: "flex-end",
                }}
              >
                <button
                  type="button"
                  onClick={() => handleApprove(false)}
                  disabled={isApproving}
                  style={{
                    padding: "8px 16px",
                    backgroundColor: "#f5f5f5",
                    border: "1px solid #e0e0e0",
                    borderRadius: "4px",
                    cursor: isApproving ? "not-allowed" : "pointer",
                    opacity: isApproving ? 0.6 : 1,
                  }}
                >
                  {isApproving ? "Approving..." : "Approve"}
                </button>
                <button
                  type="button"
                  onClick={() => handleApprove(true)}
                  disabled={isApproving}
                  style={{
                    padding: "8px 16px",
                    backgroundColor: "#2196f3",
                    color: "white",
                    border: "none",
                    borderRadius: "4px",
                    cursor: isApproving ? "not-allowed" : "pointer",
                    opacity: isApproving ? 0.6 : 1,
                  }}
                >
                  {isApproving ? "Approving..." : "Approve with Ceremony"}
                </button>
              </div>
            )}
          </motion.section>
        </motion.div>
      ) : null}
    </AnimatePresence>
  );
}
