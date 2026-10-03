final: prev:
let
  fetchSecurityPatch =
    {
      name,
      url,
      hash,
    }:
    final.fetchpatch { inherit name url hash; };

  appendSecurityPatches =
    package: securityPatches:
    package.overrideAttrs (old: {
      patches = (old.patches or [ ]) ++ map fetchSecurityPatch securityPatches;
    });

  appimageToolsWithoutLibpng12 = prev.appimageTools // {
    defaultFhsEnvArgs = prev.appimageTools.defaultFhsEnvArgs // {
      multiPkgs =
        pkgs:
        builtins.filter (package: package != pkgs.libpng12) (
          prev.appimageTools.defaultFhsEnvArgs.multiPkgs pkgs
        );
    };
  };
in
{
  ffmpeg_6 = appendSecurityPatches prev.ffmpeg_6 [
    {
      name = "ffmpeg-6-CVE-2026-64830.patch";
      url = "https://github.com/FFmpeg/FFmpeg/commit/dbd495f066a85ba96b17433f4306582aa37c3951.patch";
      hash = "sha256-13XH+2F++awIRJM8DZKE+F8hVenVfjAyMCDmAgNC8FA=";
    }
  ];

  cups-filters = appendSecurityPatches prev.cups-filters [
    {
      name = "cups-filters-CVE-2025-64524.patch";
      url = "https://github.com/OpenPrinting/cups-filters/commit/956283c74a34ae924266a2a63f8e5f529a1abd06.patch";
      hash = "sha256-+t9oNjO/ECECsH+ttlWB8Fv/bpk4QCGJrt0zGHKFQz8=";
    }
  ];

  allegro =
    let
      upstreamPatched = appendSecurityPatches prev.allegro [
        {
          name = "allegro-CVE-2021-36489-1.patch";
          url = "https://github.com/liballeg/allegro4/commit/49461ca71352c750246d6a5baf0429c6f8d52906.patch";
          hash = "sha256-h2IWs/iJRFCuyx/nZDdot2t58AV4Jeo6DCnHSwN2qKM=";
        }
        {
          name = "allegro-CVE-2021-36489-2.patch";
          url = "https://github.com/liballeg/allegro4/commit/09d6855a150e24b731dd595b73b4a4323daa3c24.patch";
          hash = "sha256-cRPNf+qZjzQKJRoY9xeIcWQLp/h1lT5VU7NeaQGIE38=";
        }
        {
          name = "allegro-CVE-2021-36489-eof-follow-up.patch";
          url = "https://github.com/liballeg/allegro4/commit/d4a10d1f2113b776b6b36742fca9f97d5708533b.patch";
          hash = "sha256-MrIMgWcw5L1iHA0IriX56yLaER17EDq1hK1zvY2kb+Q=";
        }
      ];
    in
    upstreamPatched.overrideAttrs (old: {
      patches = old.patches ++ [ ./patches/allegro-tga-control-flow.patch ];
    });

  miniaudio = appendSecurityPatches prev.miniaudio [
    {
      name = "miniaudio-CVE-2026-32837.patch";
      url = "https://github.com/mackron/miniaudio/commit/1df46ae9a0eed5aa9f58b179d2cc4af5d23f8bde.patch";
      hash = "sha256-kdGxzGb0To4s3Dwy6DoXy+42S21GSxP6eeKPV7ti06o=";
    }
  ];

  # Keep general AppImage support without adding vulnerable libpng 1.2 to its FHS.
  appimage-run = prev.appimage-run.override {
    appimageTools = appimageToolsWithoutLibpng12;
  };

  # No trusted upstream fix exists; do not add binwalk's optional DMG extractor to PATH.
  binwalk = prev.binwalk.override { dmg2img = final.emptyDirectory; };
}
