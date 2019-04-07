let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustChannel = nixpkgs.latest.rustChannels.stable;
  rustStable = rustChannel.rust.override {
    extensions = [
      "rust-src"
      "rls-preview"
      "clippy-preview"
      "rustfmt-preview"
    ];
  };
  rustML = with nixpkgs; stdenv.mkDerivation {
    name = "rust-ml";
    buildInputs = [
      cargo-edit
      rustStable
      gcc-unwrapped.lib
      openssl
      pkgconfig
      (python3.withPackages(ps: with ps; [
        ps.tensorflowWithCuda
      ]))
    ];
    shellHook = ''
      export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${nixpkgs.gcc-unwrapped.lib}/lib"
    '';
  };
in {
  inherit rustML nixpkgs;
}
