with (import <nixpkgs> {});
stdenv.mkDerivation rec {
  name = "cargo-cdo";
  buildInputs = [ rustup ];
  src = ".";
  system = builtins.currentSystem;
}
