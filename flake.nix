{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        nixpkgs-stable.follows = "nixpkgs";
      };
    };
  };

  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "riscv64-linux"
        "aarch64-darwin"
      ];

      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.pre-commit-hooks.flakeModule
      ];

      perSystem =
        {
          pkgs,
          system,
          config,
          lib,
          ...
        }:
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          devShells = {
            default =
              pkgs.mkShell.override
                (lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux { stdenv = pkgs.clangMultiStdenv; })
                rec {
                  nativeBuildInputs = with pkgs; [
                    pkg-config
                    wrapGAppsHook
                    cmake
                  ];
                  buildInputs =
                    with pkgs;
                    [
                      rust-bin.stable.latest.default
                      cargo-watch
                      cargo-tarpaulin
                      cargo-machete
                      llvmPackages.lld
                      config.treefmt.build.wrapper
                      openssl
                      zlib
                      rust-analyzer
                    ]
                    ++ (lib.optionals stdenv.hostPlatform.isLinux [
                      alsa-lib
                      wayland
                      udev
                      glib
                      gtk3
                      atk
                      libxkbcommon
                      vulkan-loader
                      dbus
                    ]);

                  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
                  CARGO_TERM_COLOR = "always";
                  shellHook = ''
                    export XDG_DATA_DIRS=$GSETTINGS_SCHEMAS_PATH
                    ${config.pre-commit.installationScript}
                  '';
                };
          };

          pre-commit = {
            check.enable = true;
            settings.hooks.treefmt = {
              enable = true;
            };
          };
          treefmt = {
            projectRootFile = "flake.lock";

            settings = {
              formatter = {
                nix = {
                  command = pkgs.nixfmt-rfc-style;
                  includes = [ "*.nix" ];
                };
                rustfmt = {
                  command = pkgs.rustfmt;
                  options = [
                    "--edition"
                    "2024"
                  ];
                  includes = [ "*.rs" ];
                };
              };
            };
          };
        };
    };
}
