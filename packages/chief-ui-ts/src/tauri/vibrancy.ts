export type ChiefVibrancyMaterial = "hud" | "popover" | "sidebar" | "under-window";

export interface ChiefVibrancyOptions {
  material?: ChiefVibrancyMaterial;
  windowLabel?: string;
  platformFallback?: "solid" | "transparent";
  invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
}

export interface ChiefVibrancyResult {
  applied: boolean;
  material: ChiefVibrancyMaterial;
  reason?: "tauri-unavailable" | "plugin-unavailable";
}

type TauriWindow = Window & {
  __TAURI__?: {
    core?: {
      invoke?: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
  };
};

function globalInvoke(): ChiefVibrancyOptions["invoke"] | undefined {
  if (typeof window === "undefined") {
    return undefined;
  }

  return (window as TauriWindow).__TAURI__?.core?.invoke;
}

export async function applyChiefVibrancy(options: ChiefVibrancyOptions = {}): Promise<ChiefVibrancyResult> {
  const material = options.material ?? "hud";
  const invoke = options.invoke ?? globalInvoke();

  if (!invoke) {
    return { applied: false, material, reason: "tauri-unavailable" };
  }

  try {
    await invoke("plugin:vibrancy|set_vibrancy", {
      label: options.windowLabel,
      material,
      fallback: options.platformFallback ?? "solid",
    });
    return { applied: true, material };
  } catch {
    return { applied: false, material, reason: "plugin-unavailable" };
  }
}
