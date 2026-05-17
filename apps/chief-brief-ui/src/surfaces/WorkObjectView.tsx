/**
 * WorkObjectView — all-day Work Object surface.
 *
 * Renders only the /v1/work/:id projection. The backend owns contribution
 * summaries and authority state; this surface groups rows for display.
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import { v1GetWorkObject } from "../api";
import type { WorkAuthorityState, WorkContribution, WorkObject } from "../types";

type WorkObjectViewProps = {
  id: string;
  onOffline?: (offline: boolean) => void;
};

const STATE_LABEL: Record<WorkAuthorityState, string> = {
  handled: "Handled",
  blocked: "Blocked",
  needs_ceremony: "Needs Ceremony",
  shipped: "Shipped",
};

const STATE_CLASS: Record<WorkAuthorityState, string> = {
  handled: "handled",
  blocked: "blocked",
  needs_ceremony: "ceremony",
  shipped: "shipped",
};

export default function WorkObjectView({ id, onOffline }: WorkObjectViewProps) {
  const [work, setWork] = useState<WorkObject | null>(null);
  const [offline, setOffline] = useState(false);

  const load = useCallback(async () => {
    try {
      const next = await v1GetWorkObject(id);
      setWork(next);
      setOffline(false);
      onOffline?.(false);
    } catch {
      setWork(null);
      setOffline(true);
      onOffline?.(true);
    }
  }, [id, onOffline]);

  useEffect(() => {
    void load();
    const timer = window.setInterval(load, 5000);
    return () => window.clearInterval(timer);
  }, [load]);

  const lanes = useMemo(() => groupByPack(work?.contributions ?? []), [work]);
  const counts = useMemo(() => countStates(work?.contributions ?? []), [work]);
  const sources = work?.source_refs ?? [];
  const provenance = work?.provenance ?? [];

  return (
    <section className="work-surface" aria-labelledby="work-title">
      <div className="work-command-row">
        <div className="work-title-block">
          <p className="eyebrow">Work Object</p>
          <h1 id="work-title">{work?.title ?? id}</h1>
          <p className="work-uri">
            {offline ? "chief-core offline" : work?.uri ?? "Syncing"}
          </p>
        </div>
        <div className="work-state-strip" aria-label="Authority state">
          <StatePill state="handled" count={counts.handled} />
          <StatePill state="needs_ceremony" count={counts.needs_ceremony} />
          <StatePill state="blocked" count={counts.blocked} />
          <StatePill state="shipped" count={counts.shipped} />
        </div>
      </div>

      <div className="work-layout">
        <div className="work-main">
          {offline ? (
            <EmptyWork title="Work Object unavailable" />
          ) : lanes.length === 0 ? (
            <EmptyWork title="No pack contributions yet" />
          ) : (
            lanes.map((lane, index) => (
              <motion.section
                className="work-lane"
                key={lane.pack}
                aria-labelledby={`work-lane-${lane.pack}`}
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: index * 0.04 }}
              >
                <div className="work-lane-header">
                  <h2 id={`work-lane-${lane.pack}`}>{lane.pack}</h2>
                  <span>{lane.items.length}</span>
                </div>
                <div className="work-contribution-grid">
                  {lane.items.map((item) => (
                    <article
                      className="work-contribution"
                      key={item.uri}
                      data-state={item.authority_state}
                    >
                      <div className="work-contribution-top">
                        <span>{item.kind.replace(/_/g, " ")}</span>
                        <span
                          className={`work-authority ${STATE_CLASS[item.authority_state]}`}
                        >
                          {STATE_LABEL[item.authority_state]}
                        </span>
                      </div>
                      <h3>{item.title}</h3>
                      {item.summary ? <p>{item.summary}</p> : null}
                      <div className="work-contribution-meta">
                        <span>{item.node_type}</span>
                        <span>{shortUri(item.uri)}</span>
                      </div>
                    </article>
                  ))}
                </div>
              </motion.section>
            ))
          )}
        </div>

        <aside className="work-side" aria-label="Sources and provenance">
          <section className="work-side-section">
            <div className="work-side-header">
              <h2>Sources</h2>
              <span>{sources.length}</span>
            </div>
            {sources.length === 0 ? (
              <p className="work-muted">No sources attached.</p>
            ) : (
              <ol className="work-source-list">
                {sources.map((source) => (
                  <li key={source.uri}>
                    <strong>{source.summary}</strong>
                    <span>{source.node_type}</span>
                    <code>{shortUri(source.uri)}</code>
                  </li>
                ))}
              </ol>
            )}
          </section>

          <section className="work-side-section">
            <div className="work-side-header">
              <h2>Provenance</h2>
              <span>{provenance.length}</span>
            </div>
            {provenance.length === 0 ? (
              <p className="work-muted">No provenance entries.</p>
            ) : (
              <ol className="work-provenance-list">
                {provenance.map((row, index) => (
                  <li key={`${row.uri ?? row.kind}-${index}`}>
                    <span>{row.kind}</span>
                    <strong>{row.source ?? "unknown"}</strong>
                    {row.uri ? <code>{shortUri(row.uri)}</code> : null}
                  </li>
                ))}
              </ol>
            )}
          </section>

          <button className="work-rewind" type="button" disabled>
            Rewind
          </button>
        </aside>
      </div>
    </section>
  );
}

function StatePill({
  state,
  count,
}: {
  state: WorkAuthorityState;
  count: number;
}) {
  return (
    <div className={`work-state-pill ${STATE_CLASS[state]}`}>
      <span>{STATE_LABEL[state]}</span>
      <strong>{count}</strong>
    </div>
  );
}

function EmptyWork({ title }: { title: string }) {
  return (
    <div className="work-empty" role="status">
      <h2>{title}</h2>
    </div>
  );
}

function groupByPack(contributions: WorkContribution[]) {
  const lanes = new Map<string, WorkContribution[]>();
  for (const item of contributions) {
    lanes.set(item.pack, [...(lanes.get(item.pack) ?? []), item]);
  }
  return Array.from(lanes.entries()).map(([pack, items]) => ({ pack, items }));
}

function countStates(contributions: WorkContribution[]) {
  return contributions.reduce(
    (acc, item) => {
      acc[item.authority_state] += 1;
      return acc;
    },
    {
      handled: 0,
      blocked: 0,
      needs_ceremony: 0,
      shipped: 0,
    } satisfies Record<WorkAuthorityState, number>
  );
}

function shortUri(uri: string): string {
  if (uri.length <= 30) return uri;
  return `${uri.slice(0, 18)}...${uri.slice(-8)}`;
}
