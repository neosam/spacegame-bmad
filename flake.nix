{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
        config.allowUnfree = true;
      };
      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = [ "wasm32-unknown-unknown" ];
      };
      bevyDeps = with pkgs; [
        vulkan-loader
        libxkbcommon
        wayland
        libdecor
        libx11
        libxcursor
        libxi
        libxrandr
        alsa-lib
        udev
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          rustToolchain
          pkgs.pkg-config
          pkgs.trunk
          pkgs.wasm-bindgen-cli
          pkgs.cargo-generate
          pkgs.claude-code
          pkgs.nodejs_24
        ] ++ bevyDeps;

        nativeBuildInputs = with pkgs; [
          pkg-config
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath bevyDeps;
      };
    };
}
