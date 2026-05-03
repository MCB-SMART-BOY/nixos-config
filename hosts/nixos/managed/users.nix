# mcbctl-managed: host-users
# mcbctl-checksum: e0846981df8d9a58cfc63afc6c4692706ff73e8ae603b799a88a3f6d251e110a
{ lib, ... }:

{
  mcb.user = lib.mkForce "mcbnixos";
  mcb.users = lib.mkForce [ "mcbnixos" ];
  mcb.adminUsers = lib.mkForce [ "mcbnixos" ];
  mcb.hostRole = lib.mkForce "desktop";
  mcb.userLinger = lib.mkForce true;
}
