{
  description = "Flake that configures Xremap, a key remapper for Linux";

  inputs = {
    # Nixpkgs will be pinned to unstable to get the latest Rust
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    # Utils for building Rust stuff
    naersk.url = "github:nmattia/naersk/master";
    # The Rust source for xremap
    xremap = {
      url = "github:k0kubun/xremap?ref=v0.4.4";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, naersk, xremap }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      naersk-lib = pkgs.callPackage naersk { };
      package = (import ./overlay xremap naersk-lib pkgs { }).xremap-unwrapped;
    in
    {
      packages."${system}".default = package;
      apps."${system}".default = {
        type = "app";
        program = "${package}/bin/xremap";
      };
      # Note, "pkgs" omitted here, so that it plays well with other overlays
      nixosModules.default = import ./modules xremap naersk-lib;
      devShells."${system}".default = with pkgs; mkShell {
        buildInputs = [ cargo rustc rustfmt rustPackages.clippy ];
        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
    };
}
