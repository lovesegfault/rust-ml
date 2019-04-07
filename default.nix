let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  rustNightlyChannel = nixpkgs.rustChannelOf { date = "2019-02-27"; channel = "nightly"; };
  rustNightly = rustNightlyChannel.rust.override {
    extensions = [
      "rust-src"
      "rls-preview"
      "clippy-preview"
      "rustfmt-preview"
    ];
  };

  daedalos = with nixpkgs;
  stdenv.mkDerivation {
    name = "rust-ml";
    buildInputs = [
      rustNightly
    ];
  };
in {
  inherit daedalos nixpkgs rustNightly;
}
