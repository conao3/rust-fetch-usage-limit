{
  description = "llm-quota development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{
      flake-parts,
      treefmt-nix,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        {
          system,
          self',
          ...
        }:
        let
          overlay = final: prev: {
            rustc = prev.rustc;
            cargo = prev.cargo;
            clippy = prev.clippy;
            rustfmt = prev.rustfmt;
          };
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ overlay ];
          };
          treefmtEval = treefmt-nix.lib.evalModule pkgs {
            projectRootFile = "flake.nix";
            programs.nixfmt.enable = true;
            programs.rustfmt.enable = true;
          };
        in
        {
          formatter = treefmtEval.config.build.wrapper;

          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "llm-quota";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };

          apps.default = {
            type = "app";
            program = "${pkgs.lib.getExe' self'.packages.default "llm-quota"}";
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
            ];
          };
        };
    };
}
