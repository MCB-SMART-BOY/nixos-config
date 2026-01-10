{ pkgs, ... }:

{
  programs.alacritty = {
    enable = true;
    settings = {
      window = {
        padding = { x = 16; y = 16; };
        decorations = "None";
        opacity = 0.95;
        blur = true;
        dynamic_padding = true;
        dynamic_title = true;
        dimensions = {
          columns = 120;
          lines = 36;
        };
      };
      scrolling = {
        history = 10000;
        multiplier = 3;
      };
      font = {
        size = 13.0;
        normal = {
          family = "JetBrainsMono Nerd Font";
          style = "Regular";
        };
        bold = {
          family = "JetBrainsMono Nerd Font";
          style = "Bold";
        };
        italic = {
          family = "JetBrainsMono Nerd Font";
          style = "Italic";
        };
        bold_italic = {
          family = "JetBrainsMono Nerd Font";
          style = "Bold Italic";
        };
        offset = { x = 0; y = 1; };
      };
      colors = {
        primary = {
          background = "#1e1e2e";
          foreground = "#cdd6f4";
          dim_foreground = "#7f849c";
          bright_foreground = "#cdd6f4";
        };
        cursor = {
          text = "#1e1e2e";
          cursor = "#f5e0dc";
        };
        vi_mode_cursor = {
          text = "#1e1e2e";
          cursor = "#b4befe";
        };
        search = {
          matches = {
            foreground = "#1e1e2e";
            background = "#a6adc8";
          };
          focused_match = {
            foreground = "#1e1e2e";
            background = "#a6e3a1";
          };
        };
        footer_bar = {
          foreground = "#1e1e2e";
          background = "#a6adc8";
        };
        hints = {
          start = {
            foreground = "#1e1e2e";
            background = "#f9e2af";
          };
          end = {
            foreground = "#1e1e2e";
            background = "#a6adc8";
          };
        };
        selection = {
          text = "#1e1e2e";
          background = "#f5e0dc";
        };
        normal = {
          black = "#45475a";
          red = "#f38ba8";
          green = "#a6e3a1";
          yellow = "#f9e2af";
          blue = "#89b4fa";
          magenta = "#f5c2e7";
          cyan = "#94e2d5";
          white = "#bac2de";
        };
        bright = {
          black = "#585b70";
          red = "#f38ba8";
          green = "#a6e3a1";
          yellow = "#f9e2af";
          blue = "#89b4fa";
          magenta = "#f5c2e7";
          cyan = "#94e2d5";
          white = "#a6adc8";
        };
        dim = {
          black = "#45475a";
          red = "#f38ba8";
          green = "#a6e3a1";
          yellow = "#f9e2af";
          blue = "#89b4fa";
          magenta = "#f5c2e7";
          cyan = "#94e2d5";
          white = "#bac2de";
        };
      };
      keyboard.bindings = [
        {
          key = "V";
          mods = "Control|Shift";
          action = "Paste";
        }
        {
          key = "C";
          mods = "Control|Shift";
          action = "Copy";
        }
        {
          key = "N";
          mods = "Control|Shift";
          action = "SpawnNewInstance";
        }
        {
          key = "Plus";
          mods = "Control";
          action = "IncreaseFontSize";
        }
        {
          key = "Minus";
          mods = "Control";
          action = "DecreaseFontSize";
        }
        {
          key = "Key0";
          mods = "Control";
          action = "ResetFontSize";
        }
      ];
      mouse.hide_when_typing = true;
      bell.duration = 0;
      cursor = {
        unfocused_hollow = true;
        blink_interval = 500;
        style = {
          shape = "Block";
          blinking = "On";
        };
        vi_mode_style = {
          shape = "Block";
          blinking = "Off";
        };
      };
      env.TERM = "xterm-256color";
    };
  };

  programs.helix = {
    enable = true;
    settings = {
      theme = "catppuccin_mocha";
      editor = {
        "line-number" = "relative";
        cursorline = true;
        cursorcolumn = false;
        "auto-save" = true;
        "auto-format" = true;
        scrolloff = 8;
        "scroll-lines" = 3;
        mouse = true;
        "true-color" = true;
        "idle-timeout" = 100;
        bufferline = "multiple";
        "color-modes" = true;
        shell = [ "zsh" "-c" ];
        "text-width" = 100;
        "default-line-ending" = "lf";

        statusline = {
          left = [ "mode" "spinner" "file-name" "read-only-indicator" "file-modification-indicator" ];
          center = [ "diagnostics" ];
          right = [ "selections" "register" "position" "file-encoding" "file-line-ending" "file-type" "version-control" ];
          separator = "│";
          mode = {
            normal = "NORMAL";
            insert = "INSERT";
            select = "SELECT";
          };
        };

        lsp = {
          enable = true;
          "display-messages" = true;
          "display-inlay-hints" = true;
          "auto-signature-help" = true;
          snippets = true;
        };

        "cursor-shape" = {
          insert = "bar";
          normal = "block";
          select = "underline";
        };

        "file-picker" = {
          hidden = false;
          "git-ignore" = true;
          "git-global" = true;
          "git-exclude" = true;
        };

        "auto-pairs" = {
          "(" = ")";
          "{" = "}";
          "[" = "]";
          "\"" = "\"";
          "`" = "`";
          "<" = ">";
        };

        search = {
          "smart-case" = true;
          "wrap-around" = true;
        };

        whitespace = {
          render = "none";
          characters = {
            space = "·";
            nbsp = "⍽";
            tab = "→";
            newline = "↵";
          };
        };

        "indent-guides" = {
          render = true;
          character = "│";
          "skip-levels" = 0;
        };

        gutters = {
          layout = [ "diagnostics" "spacer" "line-numbers" "spacer" "diff" ];
          "line-numbers" = {
            "min-width" = 3;
          };
        };

        "soft-wrap" = {
          enable = false;
          "max-wrap" = 20;
          "max-indent-retain" = 40;
          "wrap-indicator" = "↪ ";
        };

        "smart-tab" = {
          enable = true;
          "supersede-menu" = false;
        };
      };

      keys = {
        normal = {
          "C-s" = ":w";
          "C-q" = ":q";
          "C-h" = "jump_view_left";
          "C-j" = "jump_view_down";
          "C-k" = "jump_view_up";
          "C-l" = "jump_view_right";
          H = "goto_previous_buffer";
          L = "goto_next_buffer";
          "C-/" = "toggle_comments";
          y = "yank_to_clipboard";
          p = "paste_clipboard_after";
          P = "paste_clipboard_before";
          n = "search_next";
          N = "search_prev";
          g = {
            l = "goto_line_end";
            h = "goto_line_start";
            d = "goto_definition";
            r = "goto_reference";
            i = "goto_implementation";
            t = "goto_type_definition";
          };
          space = {
            f = "file_picker";
            F = "file_picker_in_current_directory";
            b = "buffer_picker";
            s = "symbol_picker";
            S = "workspace_symbol_picker";
            d = "diagnostics_picker";
            D = "workspace_diagnostics_picker";
            a = "code_action";
            r = "rename_symbol";
            h = "hover";
            "/" = "global_search";
            "?" = "command_palette";
            w = {
              v = "vsplit";
              s = "hsplit";
              q = "wclose";
              o = "wonly";
            };
            c = ":config-open";
            C = ":config-reload";
          };
        };
        insert = {
          "C-s" = ":w";
          j = { k = "normal_mode"; };
          "C-space" = "completion";
        };
        select = {
          y = "yank_to_clipboard";
        };
      };
    };

    languages = {
      "language-server" = {
        "rust-analyzer" = {
          command = "rust-analyzer";
          config = {
            inlayHints = {
              bindingModeHints.enable = false;
              closingBraceHints.minLines = 10;
              closureReturnTypeHints.enable = "with_block";
              discriminantHints.enable = "fieldless";
              lifetimeElisionHints.enable = "skip_trivial";
              typeHints.hideClosureInitialization = false;
            };
            check.command = "clippy";
            cargo = {
              allFeatures = true;
              buildScripts.enable = true;
            };
            procMacro.enable = true;
          };
        };
        nil = {
          command = "nil";
          config.nil.formatting.command = [ "nixfmt" ];
        };
        taplo = {
          command = "taplo";
          args = [ "lsp" "stdio" ];
        };
        marksman = {
          command = "marksman";
          args = [ "server" ];
        };
        "yaml-language-server" = {
          command = "yaml-language-server";
          args = [ "--stdio" ];
        };
        "bash-language-server" = {
          command = "bash-language-server";
          args = [ "start" ];
        };
        pyright = {
          command = "pyright-langserver";
          args = [ "--stdio" ];
        };
        "typescript-language-server" = {
          command = "typescript-language-server";
          args = [ "--stdio" ];
        };
        clangd = {
          command = "clangd";
          args = [ "--background-index" "--clang-tidy" ];
        };
        "lua-language-server" = {
          command = "lua-language-server";
        };
        gopls = {
          command = "gopls";
        };
      };

      language = [
        {
          name = "rust";
          scope = "source.rust";
          "injection-regex" = "rust";
          "file-types" = [ "rs" ];
          roots = [ "Cargo.toml" "Cargo.lock" ];
          "auto-format" = true;
          "language-servers" = [ "rust-analyzer" ];
          indent = {
            "tab-width" = 4;
            unit = "    ";
          };
        }
        {
          name = "nix";
          scope = "source.nix";
          "injection-regex" = "nix";
          "file-types" = [ "nix" ];
          roots = [ "flake.nix" "shell.nix" "default.nix" ];
          "comment-token" = "#";
          "language-servers" = [ "nil" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
          formatter = {
            command = "nixfmt";
          };
        }
        {
          name = "toml";
          scope = "source.toml";
          "injection-regex" = "toml";
          "file-types" = [ "toml" "Cargo.lock" ];
          roots = [ ];
          "comment-token" = "#";
          "language-servers" = [ "taplo" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
        {
          name = "markdown";
          scope = "source.markdown";
          "injection-regex" = "md|markdown";
          "file-types" = [ "md" "markdown" "mkd" "mdwn" "mkdn" "mdx" ];
          roots = [ ];
          "language-servers" = [ "marksman" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "soft-wrap".enable = true;
        }
        {
          name = "yaml";
          scope = "source.yaml";
          "file-types" = [ "yml" "yaml" ];
          roots = [ ];
          "comment-token" = "#";
          "language-servers" = [ "yaml-language-server" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
        {
          name = "bash";
          scope = "source.bash";
          "injection-regex" = "(shell|bash|zsh|sh)";
          "file-types" = [ "sh" "bash" "zsh" ".zshrc" ".bashrc" ".bash_profile" ".zshenv" ".zprofile" ];
          roots = [ ];
          "comment-token" = "#";
          "language-servers" = [ "bash-language-server" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
          formatter = {
            command = "shfmt";
            args = [ "-i" "2" ];
          };
        }
        {
          name = "python";
          scope = "source.python";
          "injection-regex" = "python";
          "file-types" = [ "py" "pyi" "py3" "pyw" "ptl" ];
          roots = [ "pyproject.toml" "setup.py" "Poetry.lock" "requirements.txt" ];
          "comment-token" = "#";
          "language-servers" = [ "pyright" ];
          indent = {
            "tab-width" = 4;
            unit = "    ";
          };
          "auto-format" = true;
          formatter = {
            command = "black";
            args = [ "--quiet" "-" ];
          };
        }
        {
          name = "typescript";
          scope = "source.ts";
          "injection-regex" = "(ts|typescript)";
          "file-types" = [ "ts" "mts" "cts" ];
          roots = [ "package.json" "tsconfig.json" ];
          "language-servers" = [ "typescript-language-server" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
        {
          name = "javascript";
          scope = "source.js";
          "injection-regex" = "(js|javascript)";
          "file-types" = [ "js" "mjs" "cjs" ];
          roots = [ "package.json" ];
          "language-servers" = [ "typescript-language-server" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
        {
          name = "c";
          scope = "source.c";
          "injection-regex" = "c";
          "file-types" = [ "c" "h" ];
          roots = [ "compile_commands.json" "Makefile" "CMakeLists.txt" ];
          "comment-token" = "//";
          "language-servers" = [ "clangd" ];
          indent = {
            "tab-width" = 4;
            unit = "    ";
          };
          "auto-format" = true;
        }
        {
          name = "cpp";
          scope = "source.cpp";
          "injection-regex" = "cpp";
          "file-types" = [ "cc" "cpp" "cxx" "c++" "hpp" "hxx" "h++" "hh" ];
          roots = [ "compile_commands.json" "Makefile" "CMakeLists.txt" ];
          "comment-token" = "//";
          "language-servers" = [ "clangd" ];
          indent = {
            "tab-width" = 4;
            unit = "    ";
          };
          "auto-format" = true;
        }
        {
          name = "lua";
          scope = "source.lua";
          "injection-regex" = "lua";
          "file-types" = [ "lua" ];
          roots = [ ".luarc.json" ".luacheckrc" "stylua.toml" ];
          "comment-token" = "--";
          "language-servers" = [ "lua-language-server" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
          formatter = {
            command = "stylua";
            args = [ "-" ];
          };
        }
        {
          name = "go";
          scope = "source.go";
          "injection-regex" = "go";
          "file-types" = [ "go" ];
          roots = [ "go.mod" "go.work" ];
          "comment-token" = "//";
          "language-servers" = [ "gopls" ];
          indent = {
            "tab-width" = 4;
            unit = "\t";
          };
          "auto-format" = true;
        }
        {
          name = "json";
          scope = "source.json";
          "injection-regex" = "json";
          "file-types" = [ "json" "jsonc" "json5" ];
          roots = [ ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
        {
          name = "html";
          scope = "text.html.basic";
          "injection-regex" = "html";
          "file-types" = [ "html" "htm" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
        {
          name = "css";
          scope = "source.css";
          "injection-regex" = "css";
          "file-types" = [ "css" ];
          indent = {
            "tab-width" = 2;
            unit = "  ";
          };
          "auto-format" = true;
        }
      ];
    };
  };
}
