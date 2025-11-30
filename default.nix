{
  pkgs ? import <nixpkgs> { },
}:

let
  rustPlatform = pkgs.rustPlatform;
  lib = pkgs.lib;
  manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = manifest.name;
  version = manifest.version;
  meta = {
    description = manifest.description;
    license = manifest.license;
    maintainers = manifest.authors;
  };

  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;

  nativeBuildInputs = [
    pkgs.gnumake
    pkgs.scdoc
  ];

  postInstall = ''
    PREFIX="$out" make -C ./docs clean install
  '';
})
