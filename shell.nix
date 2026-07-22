{ pkgs ? import <nixpkgs> { } }:

# this shell assumes you are using "rust-toolchain.toml", to specificy your rustup toolchain
# doing so ensures that people on platforms other than NixOS are also using the correct toolchain

pkgs.mkShell {
  packages = with pkgs; [
    rustup

    # cross compilation to windows
    cargo-xwin
    clang
    llvmPackages_latest.llvm
    # build a release with "cargo xwin build --release --target x86_64-pc-windows-msvc"
  ];

  # rust-analyzer tries to write to the default temp, which is read only, so we have to give it somewhere else
  # additionally ensure windows is a target (and up to date)
  shellHook = ''
    export TMPDIR=$PWD/.tmp
    mkdir -p "$TMPDIR"

    rustup target add x86_64-pc-windows-msvc
  '';
}
