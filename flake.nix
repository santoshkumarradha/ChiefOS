{
  description = "Chief OS provisional NixOS-equivalent CI environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs {
        inherit system;
      };
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
              wayland
              wayland-protocols
              libxkbcommon
            ];
          })
        ];
      };
    };
}
