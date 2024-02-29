with import (fetchTarball https://github.com/SuteraVR/nixpkgs/archive/46c06934f04dd1ac01fbcbe8c366bca2ef03ecf9.tar.gz) { };
mkShell {
  packages = [
    cargo
    rustc
    rust-analyzer
    rustfmt
    clippy
    godot4-mono
    dotnet-sdk
  ];
}
