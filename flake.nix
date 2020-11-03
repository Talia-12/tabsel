{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        package = pkgs.rustPlatform.buildRustPackage {
          pname = "tabsel";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            makeWrapper
            cmake
            pkg-config
          ];

          buildInputs = with pkgs; [
            expat
            freetype
            libX11
            libxcursor
            libxi
            libxrandr
          ];

          postFixup =
            let
              rpath = pkgs.lib.makeLibraryPath [
                pkgs.libGL
                pkgs.vulkan-loader
                pkgs.wayland
                pkgs.libxkbcommon
              ];
            in
            ''
              patchelf --set-rpath ${rpath} $out/bin/tabsel
            '';

          meta = {
            description = "A dmenu-like table selector for Wayland";
            homepage = "https://github.com/Talia-12/tabsel";
            license = pkgs.lib.licenses.mit;
            platforms = pkgs.lib.platforms.linux;
            mainProgram = "tabsel";
          };
        };
      in
      {
        packages.default = package;

        devShells.default = pkgs.mkShell {
          inputsFrom = [ package ];
          packages = with pkgs; [
            rust-analyzer
            clippy
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.libGL
            pkgs.vulkan-loader
            pkgs.wayland
            pkgs.libxkbcommon
          ];
        };
      }
    )
    // {
      homeManagerModules.default =
        {
          lib,
          pkgs,
          config,
          ...
        }:
        let
          inherit (lib)
            types
            mkIf
            mkEnableOption
            mkPackageOption
            mkOption
            ;

          cfg = config.programs.tabsel;
        in
        {
          options.programs.tabsel = {
            enable = mkEnableOption "tabsel";
            package = mkPackageOption pkgs "tabsel" {
              nullable = true;
              default = self.packages.${pkgs.system}.default;
            };
            style = mkOption {
              type = types.lines;
              default = "";
              example = ''
                .tabsel {
                  --exit-unfocused: false;
                  height: 250px;
                  width: 400px;
                  --font-family: "Iosevka,Iosevka Nerd Font";
                  font-size: 18px;
                  background: #151515;
                  color: #414141;
                  padding: 10px;

                  .container {
                    .rows {
                      --height: fill-portion 6;
                      .row-selected {
                        color: #ffffff;
                        --spacing: 3px;
                      }
                    }

                    .scrollable {
                      background: #151515;
                      width: 0;
                      .scroller {
                        width: 0;
                        color: #151515;
                      }
                    }
                  }
                }
              '';
              description = ''
                Configuration file to be written to theme.scss for setting
                Tabsel's theme.
              '';
            };
          };

          config = mkIf cfg.enable {
            assertions = [
              (lib.hm.assertions.assertPlatform "programs.tabsel" pkgs lib.platforms.linux)
            ];

            home.packages = mkIf (cfg.package != null) [ cfg.package ];
            xdg.configFile."tabsel/theme.scss" = mkIf (cfg.style != "") { text = cfg.style; };
          };
        };
    };
}
