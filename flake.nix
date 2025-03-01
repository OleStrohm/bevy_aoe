{
  description = "Rust flake with stable";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, flake-utils, rust-overlay, nixpkgs, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default.override {
          extensions = [ "rust-analyzer" "clippy" "rust-src" ];
          targets = [
            "x86_64-unknown-linux-gnu"
            "x86_64-pc-windows-gnu"
            "x86_64-pc-windows-gnullvm"
            "x86_64-pc-windows-msvc"
          ];
        };
        shellPackages = with pkgs; [
          cargo-xwin
          cargo-zigbuild
          rustToolchain
        ];

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        src = craneLib.cleanCargoSource ./.;
        nativeBuildInputs = [ rustToolchain ];
        buildInputs = with pkgs; [ 
            udev alsa-lib vulkan-loader xorg.libX11 xorg.libXcursor
            xorg.libXi xorg.libXrandr libxkbcommon wayland pkg-config
            cargo-zigbuild
        ];
        rustFlags = "-C link-args=-Wl,-rpath,${pkgs.lib.makeLibraryPath buildInputs}";
        commonArgs = {
          inherit src buildInputs nativeBuildInputs;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
        binaryName = "aoe";
      in
      with pkgs;
      {
        devShells.default = mkShell.override {
          stdenv = pkgs.stdenvAdapters.useMoldLinker clangStdenv;
        } {
          env."RUSTFLAGS" = rustFlags;
          inputsFrom = [ bin ];
        };
      }
    );
}
