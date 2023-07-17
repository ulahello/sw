{ lib, rustPlatform }:
let manifest = (lib.importTOML ./Cargo.toml).package;
in
rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;
  meta = with lib; {
    description = manifest.description;
    license = manifest.license;
    # homepage = manifest.homepage;
    maintainers = manifest.authors;
  };

  cargoLock.lockFile = ./Cargo.lock;
  src = lib.cleanSource ./.;
}
