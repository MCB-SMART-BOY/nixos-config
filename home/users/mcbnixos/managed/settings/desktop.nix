# mcbctl-managed: home-settings-desktop
# mcbctl-checksum: f7356573733f37fc83611a1ca7f1c0387acfd83c3a09dd0112be1e0650d67987
{ lib, ... }:

{
  mcb.desktopEntries.enableZed = lib.mkForce true;
  mcb.desktopEntries.enableYesPlayMusic = lib.mkForce false;
}
