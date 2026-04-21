/**
 * Offline banner shown when chief-core is not reachable.
 */

type OfflineBannerProps = {
  url: string;
};

export default function OfflineBanner({ url }: OfflineBannerProps) {
  return (
    <div
      className="offline-banner"
      style={{
        borderTop: "1px solid var(--accent-warm, #ff6b35)",
        backgroundColor: "var(--bg-1, #fafafa)",
        padding: "12px 16px",
        fontSize: "13px",
        fontWeight: 500,
        letterSpacing: "0.3px",
        color: "var(--text-secondary, #666)",
      }}
    >
      demo mode · chief-core not reachable at <code style={{ fontSize: "12px" }}>{url}</code>
    </div>
  );
}
