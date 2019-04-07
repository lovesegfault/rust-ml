let
  nixpkgs = import <nixpkgs> {};
  rustML = with nixpkgs; stdenv.mkDerivation {
    name = "rust-ml";
    buildInputs = [
      cargo
      (python3.withPackages(ps: with ps; [
        ps.tensorflowWithCuda
      ]))

    ];
  };
in {
  inherit rustML nixpkgs;
}
