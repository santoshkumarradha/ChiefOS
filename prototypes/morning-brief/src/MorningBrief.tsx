import { useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion, type Variants } from "framer-motion";
import HandledCard from "./cards/HandledCard";
import NeedsYouCard from "./cards/NeedsYouCard";
import TrustLedgerBar from "./cards/TrustLedgerBar";
import CeremonyModal from "./cards/CeremonyModal";

export type TrustCategory = {
  id: string;
  label: string;
  current: number;
  previous: number;
};

export type NeedItem = {
  id: string;
  title: string;
  category: string;
  priority: "high" | "medium" | "low";
  summary: string;
  agent: string;
  deadline: string;
  evidence: string[];
  trust_delta: number;
  estimated_minutes: number;
};

export type HandledItem = {
  id: string;
  title: string;
  category: string;
  outcome: string;
  agent: string;
  completed_at: string;
  confidence: number;
  impact: string;
  receipt: {
    source: string;
    policy: string;
    capability: string;
  };
};

export type BriefState = {
  generated_at: string;
  owner: string;
  ritual_label: string;
  needs_you: NeedItem[];
  handled: HandledItem[];
  trust_ledger: TrustCategory[];
};

const cardContainer: Variants = {
  hidden: {},
  shown: {
    transition: {
      staggerChildren: 0.075,
      delayChildren: 0.08,
    },
  },
};

const cardReveal: Variants = {
  hidden: {
    opacity: 0,
    y: 22,
    scale: 0.985,
  },
  shown: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: {
      type: "spring",
      stiffness: 360,
      damping: 31,
      mass: 0.82,
    },
  },
};

type MorningBriefProps = {
  brief: BriefState;
  isOffline?: boolean;
};

export default function MorningBrief({ brief, isOffline = false }: MorningBriefProps) {
  const [revealed, setRevealed] = useState(false);
  const [selectedHandled, setSelectedHandled] = useState<HandledItem | null>(
    null,
  );
  const [evidenceItem, setEvidenceItem] = useState<NeedItem | null>(null);
  const actionRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const receiptCloseRef = useRef<HTMLButtonElement | null>(null);

  const generatedTime = useMemo(() => {
    return new Intl.DateTimeFormat("en", {
      weekday: "short",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "2-digit",
    }).format(new Date(brief.generated_at));
  }, [brief.generated_at]);

  useEffect(() => {
    if (revealed) {
      window.setTimeout(() => actionRefs.current[0]?.focus(), 160);
    }
  }, [revealed]);

  useEffect(() => {
    if (selectedHandled) {
      receiptCloseRef.current?.focus();
    }
  }, [selectedHandled]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        if (evidenceItem) {
          setEvidenceItem(null);
          return;
        }
        if (selectedHandled) {
          setSelectedHandled(null);
          return;
        }
      }

      if (evidenceItem) {
        return;
      }

      if (event.key === " " && !revealed) {
        event.preventDefault();
        setRevealed(true);
        return;
      }

      if (
        event.key !== "ArrowDown" &&
        event.key !== "ArrowRight" &&
        event.key !== "ArrowUp" &&
        event.key !== "ArrowLeft"
      ) {
        return;
      }

      const focusable = actionRefs.current.filter(
        (node): node is HTMLButtonElement => Boolean(node),
      );
      if (focusable.length === 0) {
        return;
      }

      const currentIndex = focusable.indexOf(
        document.activeElement as HTMLButtonElement,
      );
      const direction =
        event.key === "ArrowDown" || event.key === "ArrowRight" ? 1 : -1;
      const fallbackIndex = direction === 1 ? 0 : focusable.length - 1;
      const nextIndex =
        currentIndex === -1
          ? fallbackIndex
          : (currentIndex + direction + focusable.length) % focusable.length;

      event.preventDefault();
      focusable[nextIndex]?.focus();
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [evidenceItem, revealed, selectedHandled]);

  const registerAction =
    (index: number) => (node: HTMLButtonElement | null) => {
      actionRefs.current[index] = node;
    };

  const revealBrief = () => setRevealed(true);

  return (
    <main className="brief-shell" aria-labelledby="brief-title">
      <section className="brief-hero" aria-label="Morning reveal">
        <div className="brief-title-block">
          <p className="eyebrow">{brief.ritual_label}</p>
          <h1 id="brief-title">Morning Brief</h1>
          <p className="brief-meta">
            {brief.owner} · {generatedTime}
          </p>
        </div>
        <button
          className="reveal-button"
          type="button"
          onClick={revealBrief}
          disabled={revealed}
          aria-pressed={revealed}
        >
          {revealed ? "Revealed" : "Reveal Brief"}
        </button>
      </section>

      <TrustLedgerBar categories={brief.trust_ledger} revealed={revealed} />

      <div className="brief-grid">
        <section className="needs-panel" aria-labelledby="needs-title">
          <div className="section-heading">
            <p className="section-kicker">Needs you</p>
            <h2 id="needs-title">{brief.needs_you.length} decisions</h2>
          </div>
          <motion.div
            className="card-stack"
            variants={cardContainer}
            initial="hidden"
            animate={revealed ? "shown" : "hidden"}
          >
            {brief.needs_you.map((item, index) => (
              <NeedsYouCard
                key={item.id}
                item={item}
                variants={cardReveal}
                actionRef={registerAction(index)}
                onReview={() => setEvidenceItem(item)}
                isOffline={isOffline}
              />
            ))}
          </motion.div>
        </section>

        <section className="handled-panel" aria-labelledby="handled-title">
          <div className="section-heading">
            <p className="section-kicker">Handled</p>
            <h2 id="handled-title">{brief.handled.length} receipts</h2>
          </div>
          <motion.div
            className="handled-list"
            variants={cardContainer}
            initial="hidden"
            animate={revealed ? "shown" : "hidden"}
          >
            {brief.handled.map((item, index) => (
              <HandledCard
                key={item.id}
                item={item}
                variants={cardReveal}
                actionRef={registerAction(brief.needs_you.length + index)}
                onOpen={() => setSelectedHandled(item)}
              />
            ))}
          </motion.div>
        </section>
      </div>

      <AnimatePresence>
        {selectedHandled ? (
          <motion.aside
            className="receipt-panel"
            aria-labelledby="receipt-title"
            initial={{ opacity: 0, x: 28 }}
            animate={{
              opacity: 1,
              x: 0,
              transition: { type: "spring", stiffness: 330, damping: 34 },
            }}
            exit={{
              opacity: 0,
              x: 28,
              transition: { type: "spring", stiffness: 360, damping: 34 },
            }}
          >
            <div className="receipt-header">
              <div>
                <p className="section-kicker">Receipt</p>
                <h2 id="receipt-title">{selectedHandled.title}</h2>
              </div>
              <button
                ref={receiptCloseRef}
                className="icon-button"
                type="button"
                aria-label="Close receipt"
                onClick={() => setSelectedHandled(null)}
              >
                x
              </button>
            </div>
            <dl className="receipt-facts">
              <div>
                <dt>Agent</dt>
                <dd>{selectedHandled.agent}</dd>
              </div>
              <div>
                <dt>Confidence</dt>
                <dd>{selectedHandled.confidence}%</dd>
              </div>
              <div>
                <dt>Impact</dt>
                <dd>{selectedHandled.impact}</dd>
              </div>
            </dl>
            <ol className="provenance-chain">
              <li>{selectedHandled.receipt.source}</li>
              <li>{selectedHandled.receipt.policy}</li>
              <li>{selectedHandled.receipt.capability}</li>
              <li>Decision recorded in Trust Ledger</li>
            </ol>
          </motion.aside>
        ) : null}
      </AnimatePresence>

      <CeremonyModal
        item={evidenceItem}
        onClose={() => setEvidenceItem(null)}
        isOffline={isOffline}
      />
    </main>
  );
}
