{ lib, ... }:

{
  security = {
    apparmor.enable = true;
    auditd.enable = true;
    polkit.enable = true;
    rtkit.enable = true;

    sudo = {
      enable = true;
      wheelNeedsPassword = true;
    };

    protectKernelImage = true;
  };
}
