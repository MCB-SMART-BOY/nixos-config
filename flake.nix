# 入口文件（Flake）：系统模块 + Home Manager 用户组合。
# 系统配置在 modules/，用户配置在 users/。
# hardware-configuration.nix 与 local.nix 在项目根目录（gitignored）。

{
  description = "NixOS + Home Manager configuration (unstable)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    noctalia = {
      url = "github:noctalia-dev/noctalia-shell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
      ...
    }@inputs:
    let
      defaultSystem =
        if builtins ? currentSystem then
          builtins.currentSystem
        else
          let
            envSystem = builtins.getEnv "NIX_SYSTEM";
          in
          if envSystem != "" then envSystem else "x86_64-linux";

      pkgsForDefault = nixpkgs.legacyPackages.${defaultSystem};
      overlay = import ./overlays/default.nix;

      mkNixosConfig = nixpkgs.lib.nixosSystem {
        system = defaultSystem;
        specialArgs = { inherit inputs; };
        modules = [
          ./modules
          home-manager.nixosModules.home-manager
          (
            { config, ... }:
            let
              userList = if config.mcb.users != [ ] then config.mcb.users else [ config.mcb.user ];
              mkUser =
                name:
                let
                  userModule = ./users + "/${name}/default.nix";
                in
                if builtins.pathExists userModule then
                  {
                    inherit name;
                    value = import userModule;
                  }
                else
                  throw "Missing Home Manager entry for user '${name}': expected ${toString userModule}";
            in
            {
              nixpkgs.overlays = [ overlay ];
              home-manager.useGlobalPkgs = true;
              home-manager.useUserPackages = true;
              home-manager.backupFileExtension = "bak";
              home-manager.extraSpecialArgs = { inherit inputs; };
              home-manager.users = builtins.listToAttrs (map mkUser userList);
            }
          )
        ]
        # 项目根目录的硬件配置与本地覆盖（可选）
        ++ lib.optional (builtins.pathExists ./hardware-configuration.nix) ./hardware-configuration.nix
        ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
      };
      lib = nixpkgs.lib;
    in
    {
      nixosConfigurations.host = mkNixosConfig;

      checks.${defaultSystem}.shell-syntax =
        pkgsForDefault.runCommand "shell-syntax-check"
          {
            nativeBuildInputs = with pkgsForDefault; [
              bash
              coreutils
              findutils
              gnugrep
              gnused
              shellcheck
            ];
          }
          ''
            set -euo pipefail
            cd ${self}

            check_file() {
              local file="$1"
              if [ ! -f "$file" ]; then
                return 0
              fi
              if LC_ALL=C grep -q $'\r' "$file"; then
                echo "CRLF line endings are not allowed: $file" >&2
                exit 1
              fi
              local first_line
              first_line="$(head -n 1 "$file" || true)"
              if [ "''${first_line#\#!}" != "$first_line" ]; then
                case "$first_line" in
                  *"/bash"*|*"env bash"*|*"/sh"*|*"env sh"*) ;;
                  *)
                    echo "Unsupported shebang in $file: $first_line" >&2
                    exit 1
                    ;;
                esac
                if [ ! -x "$file" ]; then
                  echo "Script with shebang must be executable: $file" >&2
                  exit 1
                fi
              fi
              if head -n 1 "$file" | grep -q '^#!'; then
                bash -n "$file"
              fi
            }

            shellcheck_file() {
              local file="$1"
              if [ ! -f "$file" ]; then
                return 0
              fi
              shellcheck -x -s bash -e SC1090,SC1091,SC2034,SC2154,SC2329 "$file"
            }

            check_file run.sh
            shellcheck_file run.sh
            bash run.sh --help >/dev/null

            while IFS= read -r -d "" file; do
              check_file "$file"
              shellcheck_file "$file"
            done < <(find users -type f -path "*/scripts/*" -print0)

            if [ -d pkgs ]; then
              while IFS= read -r -d "" file; do
                check_file "$file"
                shellcheck_file "$file"
              done < <(find pkgs -type f -path "*/scripts/*.sh" -print0)
            fi

            if [ -d scripts/run ]; then
              while IFS= read -r -d "" file; do
                if LC_ALL=C grep -q $'\r' "$file"; then
                  echo "CRLF line endings are not allowed: $file" >&2
                  exit 1
                fi
                bash -n "$file"
                shellcheck_file "$file"
              done < <(find scripts/run -type f -name "*.sh" -print0)
            fi

            touch "$out"
          '';

      formatter.${defaultSystem} = pkgsForDefault.nixfmt;
    };
}
