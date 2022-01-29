{ lib, rustPlatform, pkg-config, autoconf, alsa-lib, automake, libopus, ffmpeg, makeWrapper }:

let manifest = (lib.importTOML ./Cargo.toml).package;
in rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;

  src = lib.cleanSource ./.;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ autoconf alsa-lib automake libopus makeWrapper ffmpeg ];

  cargoLock = {
    lockFile = ./Cargo.lock;
     outputHashes = {
       "sunk-0.1.2" = "sha256-VruqNDbWbjdarXiyR1OHcXsR1MvTmCM5j+v2ZpcG5IA=";
     };
  };

  postFixup = ''
    wrapProgram $out/bin/disconic --set PATH ${lib.makeBinPath [ ffmpeg ]}
  '';

  meta = with lib; {
    description = manifest.desciption;
    homepage = manifest.homepage;
    license = licenses.agpl3Plus;
    platforms = platforms.all;
  };
}
