{ config, pkgs, ... }:

let
  wallpaperPath = "${config.xdg.configHome}/wallpaper.jpg";
  sessionTargets = [ "graphical-session.target" ];
in
{
  xdg.configFile."niri/config.kdl".source = ../config/niri/config.kdl;

  programs.swaylock = {
    enable = true;
    settings = {
      color = "1e1e2e";
      ring-color = "cba6f7";
      key-hl-color = "a6e3a1";
      line-color = "313244";
      inside-color = "1e1e2e";
      separator-color = "1e1e2e";
      ring-ver-color = "89b4fa";
      inside-ver-color = "1e1e2e";
      text-ver-color = "89b4fa";
      ring-wrong-color = "f38ba8";
      inside-wrong-color = "1e1e2e";
      text-wrong-color = "f38ba8";
      ring-clear-color = "f5e0dc";
      inside-clear-color = "1e1e2e";
      text-clear-color = "f5e0dc";
      indicator-radius = 100;
      indicator-thickness = 7;
      font = "JetBrainsMono Nerd Font";
      show-failed-attempts = true;
      daemonize = true;
      ignore-empty-password = true;
    };
  };

  programs.wofi = {
    enable = true;
    settings = {
      show = "drun";
      prompt = "➜  ";
      term = "alacritty";
      allow_images = true;
      icon_theme = "Papirus-Dark";
      lines = 10;
    };
    style = builtins.readFile ../config/wofi/style.css;
  };

  programs.waybar = {
    enable = true;
    settings = {
      mainBar = {
        layer = "top";
        position = "top";
        height = 36;
        spacing = 0;
        margin-top = 8;
        margin-left = 12;
        margin-right = 12;
        modules-left = [ "custom/logo" "niri/workspaces" ];
        modules-center = [ "clock" ];
        modules-right = [ "tray" "network" "pulseaudio" "backlight" "battery" "custom/power" ];

        "custom/logo" = {
          format = " ";
          tooltip = false;
          on-click = "wofi --show drun";
        };

        "niri/workspaces" = {
          format = "{icon}";
          "format-icons" = {
            "1" = "一";
            "2" = "二";
            "3" = "三";
            "4" = "四";
            "5" = "五";
            "6" = "六";
            "7" = "七";
            "8" = "八";
            "9" = "九";
            default = "●";
          };
          on-click = "activate";
        };

        clock = {
          format = "{:%H:%M}";
          "format-alt" = "{:%Y-%m-%d %A}";
          "tooltip-format" = "<tt><small>{calendar}</small></tt>";
          calendar = {
            mode = "month";
            "mode-mon-col" = 3;
            "weeks-pos" = "right";
            "on-scroll" = 1;
            format = {
              months = "<span color='#cba6f7'><b>{}</b></span>";
              days = "<span color='#cdd6f4'>{}</span>";
              weeks = "<span color='#94e2d5'>W{}</span>";
              weekdays = "<span color='#f9e2af'>{}</span>";
              today = "<span color='#f38ba8'><b><u>{}</u></b></span>";
            };
          };
          actions = {
            "on-click-right" = "mode";
            "on-scroll-up" = "shift_up";
            "on-scroll-down" = "shift_down";
          };
        };

        tray = {
          "icon-size" = 16;
          spacing = 8;
        };

        network = {
          "format-wifi" = "󰤨 ";
          "format-ethernet" = "󰈀 ";
          "format-disconnected" = "󰤭 ";
          "tooltip-format-wifi" = "{essid} ({signalStrength}%)\n{ipaddr}";
          "tooltip-format-ethernet" = "{ifname}\n{ipaddr}";
          "on-click" = "nm-connection-editor";
        };

        pulseaudio = {
          format = "{icon} {volume}%";
          "format-muted" = "󰝟 ";
          "format-icons" = {
            default = [ "󰕿" "󰖀" "󰕾" ];
          };
          "on-click" = "pavucontrol";
          "on-scroll-up" = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+";
          "on-scroll-down" = "wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-";
        };

        backlight = {
          format = "{icon} {percent}%";
          "format-icons" = [ "󰃞" "󰃟" "󰃠" ];
          "on-scroll-up" = "brightnessctl set 5%+";
          "on-scroll-down" = "brightnessctl set 5%-";
        };

        battery = {
          states = {
            warning = 30;
            critical = 15;
          };
          format = "{icon} {capacity}%";
          "format-charging" = "󰂄 {capacity}%";
          "format-plugged" = "󰂄 {capacity}%";
          "format-icons" = [
            "󰂎"
            "󰁺"
            "󰁻"
            "󰁼"
            "󰁽"
            "󰁾"
            "󰁿"
            "󰂀"
            "󰂁"
            "󰂂"
            "󰁹"
          ];
          "tooltip-format" = "{timeTo}\nPower: {power}W";
        };

        "custom/power" = {
          format = "⏻";
          tooltip = false;
          "on-click" = "niri msg action quit";
        };
      };
    };
    style = builtins.readFile ../config/waybar/style.css;
  };

  systemd.user.services = {
    waybar = {
      Unit = {
        Description = "Waybar status bar";
        After = sessionTargets;
        PartOf = sessionTargets;
      };
      Service = {
        ExecStart = "${pkgs.waybar}/bin/waybar";
        Restart = "on-failure";
      };
      Install.WantedBy = sessionTargets;
    };
    swaybg = {
      Unit = {
        Description = "Wayland wallpaper";
        After = sessionTargets;
        PartOf = sessionTargets;
      };
      Service = {
        ExecStart = "${pkgs.swaybg}/bin/swaybg -i ${wallpaperPath} -m fill";
        Restart = "on-failure";
      };
      Install.WantedBy = sessionTargets;
    };
    swayidle = {
      Unit = {
        Description = "Wayland idle manager";
        After = sessionTargets;
        PartOf = sessionTargets;
      };
      Service = {
        ExecStart = "${pkgs.swayidle}/bin/swayidle -w";
        Restart = "on-failure";
      };
      Install.WantedBy = sessionTargets;
    };
    fcitx5 = {
      Unit = {
        Description = "Fcitx5 input method";
        After = sessionTargets;
        PartOf = sessionTargets;
      };
      Service = {
        ExecStart = "${pkgs.fcitx5}/bin/fcitx5 -r";
        Restart = "on-failure";
      };
      Install.WantedBy = sessionTargets;
    };
  };

  services.dunst = {
    enable = true;
    iconTheme = {
      package = pkgs.papirus-icon-theme;
      name = "Papirus-Dark";
      size = "48x48";
    };
    settings = {
      global = {
        font = "JetBrainsMono Nerd Font 12";
        width = 400;
        height = 150;
        offset = "20x20";
        origin = "top-right";
        padding = 15;
        frame_width = 2;
        frame_color = "#cba6f7";
        separator_color = "#cba6f7";
        background = "#1e1e2e";
        foreground = "#cdd6f4";
        corner_radius = 12;
        timeout = 5;
        layer = "overlay";
        max_icon_size = 48;
        icon_position = "left";
        show_age_threshold = 60;
      };
      urgency_low = {
        background = "#1e1e2e";
        foreground = "#cdd6f4";
        frame_color = "#a6e3a1";
      };
      urgency_normal = {
        background = "#1e1e2e";
        foreground = "#cdd6f4";
        frame_color = "#cba6f7";
      };
      urgency_critical = {
        background = "#1e1e2e";
        foreground = "#cdd6f4";
        frame_color = "#f38ba8";
        timeout = 0;
      };
    };
  };

  gtk = {
    enable = true;
    theme = {
      name = "Adwaita-dark";
      package = pkgs.gnome-themes-extra;
    };
    iconTheme = {
      name = "Papirus-Dark";
      package = pkgs.papirus-icon-theme;
    };
    cursorTheme = {
      name = "Bibata-Modern-Classic";
      package = pkgs.bibata-cursors;
      size = 24;
    };
    font = {
      name = "Noto Sans CJK SC 11";
    };
    gtk3.extraConfig = {
      gtk-application-prefer-dark-theme = 1;
    };
    gtk4.extraConfig = {
      gtk-application-prefer-dark-theme = 1;
      gtk-font-name = "Noto Sans CJK SC 12";
    };
  };
}
