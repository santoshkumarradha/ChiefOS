import { motion } from "framer-motion";
import type { TrustCategory } from "../MorningBrief";

type TrustLedgerBarProps = {
  categories: TrustCategory[];
  revealed: boolean;
};

export default function TrustLedgerBar({
  categories,
  revealed,
}: TrustLedgerBarProps) {
  return (
    <section className="trust-ledger" aria-labelledby="trust-ledger-title">
      <div className="section-heading compact">
        <p className="section-kicker">Trust ledger</p>
        <h2 id="trust-ledger-title">Delegation health</h2>
      </div>
      <div className="trust-bars">
        {categories.map((category) => {
          const value = revealed ? category.current : category.previous;
          const delta = category.current - category.previous;

          return (
            <article className="trust-row" key={category.id}>
              <div className="trust-label">
                <span>{category.label}</span>
                <strong>{value}%</strong>
              </div>
              <div className="trust-track" aria-hidden="true">
                <motion.div
                  className="trust-fill"
                  initial={{ width: `${category.previous}%` }}
                  animate={{ width: `${value}%` }}
                  transition={{
                    type: "spring",
                    stiffness: 170,
                    damping: 24,
                    mass: 0.72,
                  }}
                />
              </div>
              <span className={delta >= 0 ? "trust-delta good" : "trust-delta"}>
                {delta >= 0 ? "+" : ""}
                {delta}
              </span>
            </article>
          );
        })}
      </div>
    </section>
  );
}
