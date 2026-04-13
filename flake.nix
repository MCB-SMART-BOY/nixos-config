# 入口文件（Flake）：定义依赖、主机列表与 Home Manager 组合方式。
# 新增/删除主机时，先看这里如何扫描 hosts/。

{
  description = "NixOS + Home Manager configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    nixpkgs-24_11.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager/release-25.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    noctalia = {
      url = "github:noctalia-dev/noctalia-shell";
      inputs.nixpkgs.follows = "nixpkgs-unstable";
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
      # 当前系统架构（优先使用 currentSystem；不支持时回退到 NIX_SYSTEM 或 x86_64-linux）
      defaultSystem =
        if builtins ? currentSystem then
          builtins.currentSystem
        else
          let
            envSystem = builtins.getEnv "NIX_SYSTEM";
          in
          if envSystem != "" then envSystem else "x86_64-linux";
      # 内置默认架构仅用于已知主机；新增主机请显式提供 hosts/<name>/system.nix。
      hostSystemDefaults = {
        laptop = "x86_64-linux";
        nixos = "x86_64-linux";
        server = "x86_64-linux";
      };
      # 每个 host 应显式指定 system（hosts/<name>/system.nix），避免跨架构误评估/误构建。
      hostSystem =
        name:
        let
          systemFile = ./hosts + "/${name}/system.nix";
        in
        if builtins.pathExists systemFile then
          import systemFile
        else if builtins.hasAttr name hostSystemDefaults then
          hostSystemDefaults.${name}
        else
          throw "Missing hosts/${name}/system.nix. Please define an explicit target system (e.g. \"x86_64-linux\").";
      # 自动读取 hosts/ 下的真实主机目录（排除 profiles / templates）
      hostEntries = builtins.readDir ./hosts;
      hostNames = builtins.filter (
        name:
        hostEntries.${name} == "directory"
        && !(builtins.elem name [
          "profiles"
          "templates"
        ])
      ) (builtins.attrNames hostEntries);
      overlay =
        final: prev:
        let
          unstablePkgs = import inputs.nixpkgs-unstable {
            system = final.stdenv.hostPlatform.system;
          };
        in
        {
          valkey = prev.valkey.overrideAttrs (old: {
            doCheck = false;
          });
          gridix = final.callPackage ./pkgs/gridix { inherit unstablePkgs; };
          mcbctl = final.callPackage ./pkgs/mcbctl { };
        };
      pkgsForDefault = import nixpkgs {
        system = defaultSystem;
        overlays = [ overlay ];
      };
      mcbctlPkg = pkgsForDefault.mcbctl;
      # 为每个主机构造 nixosSystem，并注入 Home Manager
      mkHost =
        name:
        nixpkgs.lib.nixosSystem {
          system = hostSystem name;
          specialArgs = { inherit inputs; };
          modules = [
            (./hosts + "/${name}")
            home-manager.nixosModules.home-manager
            (
              { config, pkgs, ... }:
              let
                # 从 mcb.users 收集所有用户，否则使用单一 mcb.user
                userList = if config.mcb.users != [ ] then config.mcb.users else [ config.mcb.user ];
                # 将每个用户映射到 home/users/<name>
                mkUser =
                  name:
                  let
                    userModule = ./home/users + "/${name}/default.nix";
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
                # Home Manager 与系统共享同一套 pkgs
                home-manager.useGlobalPkgs = true;
                home-manager.useUserPackages = true;
                home-manager.backupFileExtension = "bak";
                home-manager.extraSpecialArgs = { inherit inputs; };
                # 批量启用 Home Manager 用户
                home-manager.users = builtins.listToAttrs (map mkUser userList);
              }
            )
          ];
        };
    in
    {
      nixosConfigurations = builtins.listToAttrs (
        map (name: {
          inherit name;
          value = mkHost name;
        }) hostNames
      );

      packages.${defaultSystem} = {
        default = mcbctlPkg;
        mcbctl = mcbctlPkg;
        gridix = pkgsForDefault.gridix;
      };

      apps.${defaultSystem} =
        let
          mkApp = program: {
            type = "app";
            inherit program;
            meta.description = "Rust script entry for this NixOS configuration";
          };
          commandApps = [
            "mcbctl"
            "mcb-deploy"
            "deploy"
            "clash-verge-prestart"
            "mcb-tun-route"
            "lock-screen"
            "niri-run"
            "steam-gamescope"
            "wallpaper-random"
            "electron-auto-gpu"
            "zed-auto-gpu"
            "flatpak-setup"
            "musicfox-wrapper"
            "noctalia-bluetooth"
            "noctalia-cpu"
            "noctalia-disk"
            "noctalia-flake-updates"
            "noctalia-gpu-current"
            "noctalia-gpu-mode"
            "noctalia-memory"
            "noctalia-net-speed"
            "noctalia-net-status"
            "noctalia-power"
            "noctalia-proxy-status"
            "noctalia-temperature"
            "update-zed-source"
            "update-yesplaymusic-source"
            "update-upstream-apps"
          ];
        in
        builtins.listToAttrs (
          [
            {
              name = "default";
              value = mkApp "${mcbctlPkg}/bin/mcbctl";
            }
          ]
          ++ map (name: {
            inherit name;
            value = mkApp "${mcbctlPkg}/bin/${name}";
          }) commandApps
        );

      checks.${defaultSystem} = {
        rust-repo-integrity =
          pkgsForDefault.runCommand "rust-repo-integrity"
            {
              nativeBuildInputs = [ mcbctlPkg ];
            }
            ''
              ${mcbctlPkg}/bin/mcbctl lint-repo --root ${self}
              touch "$out"
            '';

        mcbctl-build = mcbctlPkg;
      };

      formatter.${defaultSystem} = pkgsForDefault.nixfmt;
    };
}
