{ ... }:

{
  security = {
    apparmor.enable = true;
    audit = {
      enable = true;
      rules = [
        "-a never,exclude -F msgtype=PROCTITLE"
        "-a always,exit -F arch=b64 -S mount -S umount2 -k mount-changes"
        "-a always,exit -F arch=b64 -S init_module -S finit_module -S delete_module -k kernel-modules"
        "-a always,exit -F arch=b64 -S sethostname -S setdomainname -k host-identity"
        "-a always,exit -F arch=b64 -S clock_settime -S adjtimex -k system-clock"
      ];
    };
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
