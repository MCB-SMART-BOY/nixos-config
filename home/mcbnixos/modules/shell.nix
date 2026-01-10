{ ... }:

{
  programs.zsh = {
    enable = true;
    enableCompletion = true;
    autosuggestion.enable = true;
    syntaxHighlighting.enable = true;
    oh-my-zsh = {
      enable = true;
      plugins = [
        "git"
        "sudo"
        "docker"
        "rust"
        "fzf"
      ];
      theme = "robbyrussell";
    };
    initExtra = builtins.readFile ../config/zsh/extra.zsh;
  };

  programs.direnv = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.zoxide = {
    enable = true;
    enableZshIntegration = true;
  };

  programs.fzf = {
    enable = true;
    enableZshIntegration = true;
    defaultCommand = "fd --type f --hidden --follow --exclude .git";
    changeDirWidgetCommand = "fd --type d --hidden --follow --exclude .git";
    defaultOptions = [
      "--height=40%"
      "--layout=reverse"
      "--border=rounded"
      "--preview-window=right:60%"
      "--color=bg+:#313244,bg:#1e1e2e,spinner:#f5e0dc,hl:#f38ba8"
      "--color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc"
      "--color=marker:#f5e0dc,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8"
    ];
  };

  programs.starship = {
    enable = true;
    enableZshIntegration = false;
    settings = {
      format = ''
$os\
$username\
$hostname\
$directory\
$git_branch\
$git_status\
$git_commit\
$rust\
$python\
$golang\
$nodejs\
$nix_shell\
$docker_context\
$cmd_duration\
$line_break\
$character'';
      command_timeout = 1000;
      add_newline = true;
      palette = "catppuccin_mocha";

      character = {
        success_symbol = "[❯](bold mauve)";
        error_symbol = "[❯](bold red)";
        vimcmd_symbol = "[❮](bold green)";
        vimcmd_replace_symbol = "[❮](bold purple)";
        vimcmd_replace_one_symbol = "[❮](bold purple)";
        vimcmd_visual_symbol = "[❮](bold yellow)";
      };

      os = {
        disabled = false;
        style = "bold blue";
        format = "[$symbol]($style) ";
        symbols = {
          NixOS = " ";
          Linux = " ";
          Macos = " ";
          Windows = " ";
        };
      };

      username = {
        show_always = false;
        style_user = "bold lavender";
        style_root = "bold red";
        format = "[$user]($style)@";
      };

      hostname = {
        ssh_only = true;
        style = "bold green";
        format = "[$hostname]($style) ";
      };

      directory = {
        style = "bold sky";
        format = "[$path]($style)[$read_only]($read_only_style) ";
        truncation_length = 4;
        truncation_symbol = "…/";
        truncate_to_repo = true;
        read_only = " 󰌾";
        read_only_style = "bold red";
        substitutions = {
          Documents = "󰈙 ";
          Downloads = " ";
          Music = "󰝚 ";
          Pictures = " ";
          Projects = "󰲋 ";
          "~" = "󰋜 ";
        };
      };

      git_branch = {
        symbol = " ";
        style = "bold mauve";
        format = "on [$symbol$branch(:$remote_branch)]($style) ";
      };

      git_status = {
        style = "bold maroon";
        format = "([\\[$all_status$ahead_behind\\]]($style) )";
        ahead = "⇡\${count}";
        behind = "⇣\${count}";
        diverged = "⇕⇡\${ahead_count}⇣\${behind_count}";
        conflicted = "=";
        untracked = "?";
        stashed = "$";
        modified = "!";
        staged = "+";
        renamed = "»";
        deleted = "✘";
      };

      git_commit = {
        style = "bold green";
        format = "[(\\($hash$tag\\))]($style) ";
        commit_hash_length = 7;
        tag_disabled = false;
        tag_symbol = " 🏷  ";
        only_detached = true;
      };

      rust = {
        symbol = "🦀 ";
        style = "bold peach";
        format = "via [$symbol($version )]($style)";
        detect_extensions = [ "rs" ];
        detect_files = [ "Cargo.toml" "Cargo.lock" ];
      };

      python = {
        symbol = " ";
        style = "bold yellow";
        format = "via [\\${symbol}\\${pyenv_prefix}(\\${version} )(\\(\\${virtualenv}\\) )]($style)";
        detect_extensions = [ "py" ];
        detect_files = [ "requirements.txt" "pyproject.toml" "setup.py" "Pipfile" ];
      };

      golang = {
        symbol = " ";
        style = "bold cyan";
        format = "via [$symbol($version )]($style)";
        detect_extensions = [ "go" ];
        detect_files = [ "go.mod" "go.sum" ];
      };

      nodejs = {
        symbol = " ";
        style = "bold green";
        format = "via [$symbol($version )]($style)";
        detect_extensions = [ "js" "ts" "mjs" ];
        detect_files = [ "package.json" "tsconfig.json" ];
      };

      nix_shell = {
        symbol = " ";
        style = "bold blue";
        impure_msg = "[impure](bold red)";
        pure_msg = "[pure](bold green)";
        format = "via [$symbol$state( \\($name\\))]($style) ";
      };

      docker_context = {
        symbol = " ";
        style = "bold blue";
        format = "via [$symbol$context]($style) ";
        only_with_files = true;
        detect_files = [ "docker-compose.yml" "docker-compose.yaml" "Dockerfile" ];
      };

      cmd_duration = {
        min_time = 2000;
        style = "bold yellow";
        format = "took [$duration]($style) ";
        show_milliseconds = false;
        show_notifications = false;
      };

      palettes.catppuccin_mocha = {
        rosewater = "#f5e0dc";
        flamingo = "#f2cdcd";
        pink = "#f5c2e7";
        mauve = "#cba6f7";
        red = "#f38ba8";
        maroon = "#eba0ac";
        peach = "#fab387";
        yellow = "#f9e2af";
        green = "#a6e3a1";
        teal = "#94e2d5";
        sky = "#89dceb";
        sapphire = "#74c7ec";
        blue = "#89b4fa";
        lavender = "#b4befe";
        text = "#cdd6f4";
        subtext1 = "#bac2de";
        subtext0 = "#a6adc8";
        overlay2 = "#9399b2";
        overlay1 = "#7f849c";
        overlay0 = "#6c7086";
        surface2 = "#585b70";
        surface1 = "#45475a";
        surface0 = "#313244";
        base = "#1e1e2e";
        mantle = "#181825";
        crust = "#11111b";
      };
    };
  };
}
