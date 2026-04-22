{
  description = "Chief OS provisional NixOS-equivalent CI environment with Hyprland compositor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    
    # Hyprland pinned to stable release
    # v0.49.0 released 2025-03-10, stable and well-tested
    hyprland = {
      type = "github";
      owner = "hyprwm";
      repo = "Hyprland";
      rev = "refs/tags/v0.49.0";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, hyprland }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs {
        inherit system;
      };
      
      # Build Hyprland from the pinned source
      hyprlandFor = system:
        let
          pkgs = pkgsFor system;
        in
        if system == "x86_64-linux" || system == "aarch64-linux" then
          # Hyprland is Linux-only; use nixpkgs hyprland for non-Linux systems
          pkgs.hyprland.overrideAttrs (old: {
            src = hyprland;
            version = "0.49.0";
          })
        else
          pkgs.hyprland;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              fuse
              fuse3
              libbpf
              pkg-config
              rustc
              rustfmt
              systemd
              wayland
              wayland-protocols
              libxkbcommon
            ];
          };
        });

      nixosConfigurations.chief-os-ci = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          # Import chief-compositor module
          ./nix/chief-compositor.nix
          
          # Base configuration
          ({ pkgs, ... }: {
            system.stateVersion = "25.11";
            networking.hostName = "chief-os-ci";
            nix.settings.experimental-features = [ "nix-command" "flakes" ];
            
            environment.systemPackages = with pkgs; [
              cargo
              fuse
              fuse3
              libbpf
              pkg-config
              rustc
              systemd
              # systemd-nspawn binary used by chief-core's sandbox module
              # (see crates/chief-core/src/sandbox/nspawn.rs and ADR-0002).
              systemd-container
              wayland
              wayland-protocols
              libxkbcommon
            ];
            
            # Enable Chief OS compositor (Hyprland)
            chief.compositor.enable = true;
            chief.compositor.blurIntensity = 0.6;
            # Use the pinned Hyprland version
            chief.compositor.package = hyprlandFor "x86_64-linux";
          })
        ];
      };
    };
}
