/**
 * App — Chief OS demo shell.
 *
 * Surfaces:
 *   - Menubar (always visible)
 *   - WorkObjectView (all-day demo center)
 *   - MorningBrief (ritual surface, mounted underneath)
 *   - InboxDrawer (⌘I)
 *   - Omnibar (⌘Space)
 *   - Ceremony (overlay; auto-opens when the inbox stream surfaces a
 *     CeremonyPending item the user hasn't seen yet, or when an inbox row
 *     is clicked)
 *
 * All surfaces are overlays. MorningBrief stays mounted underneath.
 */

import { useEffect, useState } from "react";
import Menubar from "./surfaces/Menubar";
import WorkObjectView from "./surfaces/WorkObjectView";
import MorningBrief from "./MorningBrief";
import InboxDrawer from "./surfaces/InboxDrawer";
import Omnibar from "./surfaces/Omnibar";
import Ceremony from "./surfaces/Ceremony";
import { useInboxStream } from "./hooks/useInboxStream";
import { useKeyboard } from "./hooks/useKeyboard";

export default function App() {
  const [inboxOpen, setInboxOpen] = useState(false);
  const [omnibarOpen, setOmnibarOpen] = useState(false);
  const [activeCeremony, setActiveCeremony] = useState<string | null>(null);
  const [briefOffline, setBriefOffline] = useState(false);
  const [workOffline, setWorkOffline] = useState(false);

  // Listen to the inbox stream at shell level too, so we can auto-promote a
  // new pending ceremony regardless of whether the drawer is open. The drawer
  // itself also uses this hook (React stays fine with multiple consumers; each
  // gets its own EventSource — acceptable for this prototype).
  const { ceremonyPending } = useInboxStream();
  const [seenCeremonies, setSeenCeremonies] = useState<Set<string>>(
    () => new Set()
  );

  useEffect(() => {
    if (!ceremonyPending || !ceremonyPending.ceremony_id) return;
    const cid = ceremonyPending.ceremony_id;
    if (seenCeremonies.has(cid)) return;
    setSeenCeremonies((prev) => new Set(prev).add(cid));
    setActiveCeremony(cid);
  }, [ceremonyPending, seenCeremonies]);

  useKeyboard({
    "Meta+i": () => setInboxOpen((v) => !v),
    "Meta+Space": () => setOmnibarOpen((v) => !v),
    Escape: () => {
      // Dismiss surfaces from top → bottom: ceremony > omnibar > inbox.
      if (activeCeremony) {
        setActiveCeremony(null);
        return;
      }
      if (omnibarOpen) {
        setOmnibarOpen(false);
        return;
      }
      if (inboxOpen) {
        setInboxOpen(false);
      }
    },
  });

  const handleCeremonyOpen = (id: string) => {
    setActiveCeremony(id);
    setInboxOpen(false);
  };

  return (
    <div className="shell-root">
      <Menubar />
      <WorkObjectView id="acme-follow-up" onOffline={setWorkOffline} />
      <MorningBrief
        onCeremonyClick={handleCeremonyOpen}
        onOffline={setBriefOffline}
      />
      <InboxDrawer
        open={inboxOpen}
        onClose={() => setInboxOpen(false)}
        onCeremonyClick={handleCeremonyOpen}
      />
      <Omnibar open={omnibarOpen} onClose={() => setOmnibarOpen(false)} />
      {activeCeremony ? (
        <Ceremony
          id={activeCeremony}
          onDone={() => setActiveCeremony(null)}
        />
      ) : null}

      {briefOffline || workOffline ? (
        <div
          className="shell-toast tone-warn"
          role="status"
          aria-live="polite"
        >
          chief-core offline — surfaces will populate when reachable.
        </div>
      ) : null}
    </div>
  );
}
