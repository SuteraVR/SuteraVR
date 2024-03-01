let
  rust_overlay = import (builtins.fetchTarball https://github.com/oxalica/rust-overlay/archive/72fa0217f76020ad3aeb2dd9dd72490905b23b6f.tar.gz);
  pkgs = import (builtins.fetchTarball https://github.com/SuteraVR/nixpkgs/archive/46c06934f04dd1ac01fbcbe8c366bca2ef03ecf9.tar.gz) {
    overlays = [
      rust_overlay
    ];
  };
  rustVersion = "1.74.0";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
    ];
  };
in
pkgs.mkShell {
  packages = with pkgs; [
    rust
    godot4-mono
    dotnet-sdk
  ];
}
