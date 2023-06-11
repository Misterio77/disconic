{ lib, rustPlatform, pkg-config, autoconf, alsa-lib, automake, libopus, ffmpeg, makeWrapper }:

let manifest = (lib.importTOML ./Cargo.toml).package;
in rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;

  src = lib.cleanSource ./.;

  nativeBuildInputs = [ pkg-config autoconf automake makeWrapper ];
  buildInputs = [ alsa-lib libopus ffmpeg ];

  cargoLock = {
    lockFile = ./Cargo.lock;
     outputHashes = {
       "sunk-0.1.2" = "sha256-gxDinyzKHxph2D5OZ9TevVEWtsEnnFpu4BfkkBnRMx4=";
       "serenity-0.11.5" = "sha256-10s0kflNYEMwUXAgrh6d1IUk3ZRSCkAilz9m1lVhXhA=";
       "songbird-0.3.2" = "sha256-8wzCcV9W6K0MHqZ8yhTIMjh165NV8OQ9zlgrRrIhlOI=";
       "poise-0.4.1" = "sha256-MPfg3miu9tesGKChaR8vz8RZA45I5uJs60MtgxDJryw=";
     };
  };

  postFixup = ''
    wrapProgram $out/bin/disconic --set PATH ${lib.makeBinPath [ ffmpeg ]}
  '';

  meta = with lib; {
    description = manifest.description;
    homepage = manifest.homepage;
    license = licenses.agpl3Plus;
    platforms = platforms.all;
  };
}
