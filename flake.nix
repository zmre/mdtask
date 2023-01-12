{
  description = "mdtask flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, naersk, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rusttoolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        naersk-lib = naersk.lib."${system}";
      in rec {
        # `nix build`
        packages.mdtask = naersk-lib.buildPackage {
          pname = "mdtask";
          root = ./.;
        };
        defaultPackage = packages.mdtask;

        # `nix run`
        apps.mdtask = flake-utils.lib.mkApp { drv = packages.mdtask; };
        defaultApp = apps.mdtask;

        # nix develop
        devShell = pkgs.mkShell {
          buildInputs = with pkgs;
            [
              #openssl
              #pkg-config
              cargo-watch
              rusttoolchain
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs;
              [
                # libiconv
                darwin.apple_sdk.frameworks.Security
              ]);
        };
      });
}
