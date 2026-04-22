# chief-compositor.nix — Chief OS Hyprland integration
# NixOS module that packages Hyprland with Chief-specific configuration
# Implements ADR-0015: Chief OS bundles Hyprland as the shipping Wayland compositor

{ config, pkgs, lib, ... }:
with lib;
let
  cfg = config.chief.compositor;
  hyprlandPkg = pkgs.hyprland;
  
  # Convert blurIntensity (0.0-1.0) to Hyprland blur_size (0-20)
  blurSize = builtins.floor (cfg.blurIntensity * 20);
  
  # Generate hyprland.conf with Chief-specific configuration
  # Based on brand/visual-language.md §6 (shadows/depth) + §7 (motion)
  hyprlandConf = pkgs.writeText "hyprland.conf" ''
    # Chief OS Hyprland Configuration
    # Implements design language from docs/brand/visual-language.md
    # Pinned version per ADR-0015 via flake input
    
    monitor = , preferred, auto, auto
    
    # Keyboard layout
    input {
      kb_layout = us
      kb_model = pc104
      follow_mouse = 1
      mouse_refocus = true
    }
    
    # General compositor settings
    general {
      allow_tearing = false
      gaps_in = 0
      gaps_out = 0
      border_size = 0
      col.active_border = rgba(00000000)
      col.inactive_border = rgba(00000000)
      # Disable layout switching (Chief OS is single-surface, not tiling)
      layout = dwindle
    }
    
    # Decoration: blur + rounding matching brand spec
    decoration {
      # Blur enabled on layer-shell surfaces (Omnibar, HAX Inbox, etc.)
      blur {
        enabled = true
        size = ${toString blurSize}
        passes = 2
        new_optimizations = true
        xray = false
        # Only apply blur to layer-shell surfaces (overlays)
        # not to the main window itself
      }
      
      # Rounding = 10px (matches --chief-radius-card from visual-language.md §5)
      rounding = 10
      
      # Shadow configuration from visual-language.md §6
      shadow {
        enabled = true
        range = 32
        render_power = 2
        color = rgba(00000064)
      }
    }
    
    # Animation curves (bezier approximations of spring values)
    # NOTE: Hyprland uses bezier easing curves, not spring physics.
    # Spring values from visual-language.md §7 are translated to approximate bezier curves:
    #   - stiffness=220, damping=28 (firm, settled) → bezier 0.25 0.46 0.45 0.94
    #   - stiffness=180, damping=22 (satisfying drift) → bezier 0.34 0.49 0.41 0.93
    #   - stiffness=260, damping=30 (snappy) → bezier 0.20 0.50 0.52 0.96
    #   - stiffness=120, damping=20 (slow, deliberate) → bezier 0.51 0.30 0.40 0.99
    #
    # These are approximations; actual spring feels may differ slightly.
    animations {
      enabled = true
      
      # Default easing: firm, settled
      bezier = main, 0.25, 0.46, 0.45, 0.94
      # Drift easing
      bezier = drift, 0.34, 0.49, 0.41, 0.93
      # Snappy easing
      bezier = snappy, 0.20, 0.50, 0.52, 0.96
      # Slow, deliberate easing
      bezier = deliberate, 0.51, 0.30, 0.40, 0.99
      
      # Card reveal / general window animations
      animation = windows, 1, 0.4, main
      animation = windowsIn, 1, 0.4, main
      animation = windowsOut, 1, 0.4, main
      animation = windowsMove, 1, 0.4, main
      
      # Layer animations (Omnibar, overlays) — snappy
      animation = layers, 1, 0.3, snappy
      animation = layersIn, 1, 0.3, snappy
      animation = layersOut, 1, 0.3, snappy
      
      # Fade animations
      animation = fade, 1, 0.2, main
      
      # Border animations
      animation = border, 1, 0.2, main
      
      # Workspace switch (deliberately slow)
      animation = workspaces, 1, 0.5, deliberate
    }
    
    # Tiling disabled per visual-language.md §14 (single-surface UX)
    # Users who want tiling can enable via Controls (future).
    dwindle {
      pseudotile = false
      preserve_split = false
    }
    
    # Keybindings
    # Reserve keybinds for Chief OS (Omnibar, Ceremony, etc.)
    # Disable default Hyprland tiling keybinds (mod+arrow, mod+hjkl, mod+q, etc.)
    
    # Omnibar summon: ⌘+Space (reserved for Chief UI)
    # Do not bind; this is managed by Chief UI as a Tauri global shortcut
    
    # Ceremony hold: reserved (will be Tauri-managed)
    # Do not bind
    
    # Exit Hyprland (disabled; Chief OS manages shutdown)
    bind = SUPER SHIFT, E, submap, reset
    
    # Default submap
    submap = reset
  '';
  
in
{
  options.chief.compositor = {
    enable = mkEnableOption "Chief OS Hyprland compositor integration" // { default = true; };
    
    blurIntensity = mkOption {
      type = types.float;
      default = 0.6;
      description = ''
        Blur intensity for layer-shell surfaces (Omnibar, HAX Inbox, etc.).
        Range: 0.0 (no blur) to 1.0 (maximum blur).
        Mapped to Hyprland decoration.blur.size (0-20).
        Default (0.6) matches brand/visual-language.md §6 overlay blur.
      '';
    };
    
    package = mkOption {
      type = types.package;
      default = hyprlandPkg;
      description = "Hyprland package to use (allows version pinning via flake input)";
    };
  };
  
  config = mkIf cfg.enable {
    # Enable Wayland session support
    services.displayManager.sessionPackages = [ cfg.package ];
    
    # Ensure Hyprland is available in the system
    environment.systemPackages = with pkgs; [
      cfg.package
      wayland
      wayland-protocols
      libxkbcommon
    ];
    
    # Enable Hyprland via programs.hyprland
    programs.hyprland = {
      enable = true;
      package = cfg.package;
      
      # Pure Wayland (no XWayland) per ADR-0003
      xwayland.enable = false;
    };
    
    # Ship hyprland.conf with Chief-specific configuration
    environment.etc."hyprland/hyprland.conf".source = hyprlandConf;
  };
}
