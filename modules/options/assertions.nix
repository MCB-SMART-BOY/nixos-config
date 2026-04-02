# mcb.* 选项的基础一致性校验。

{ lib, config, ... }:

{
  config.assertions = [
    {
      assertion = (lib.length config.mcb.users == 0) || (lib.elem config.mcb.user config.mcb.users);
      message = "mcb.user must be included in mcb.users when mcb.users is set.";
    }
    {
      assertion = lib.length config.mcb.users == lib.length (lib.unique config.mcb.users);
      message = "mcb.users must not contain duplicate entries.";
    }
    {
      assertion =
        (lib.length config.mcb.adminUsers == 0)
        || (lib.all (
          user: lib.elem user (if config.mcb.users != [ ] then config.mcb.users else [ config.mcb.user ])
        ) config.mcb.adminUsers);
      message = "mcb.adminUsers must be a subset of managed users (mcb.users or mcb.user).";
    }
    {
      assertion = lib.length config.mcb.adminUsers == lib.length (lib.unique config.mcb.adminUsers);
      message = "mcb.adminUsers must not contain duplicate entries.";
    }
  ];
}
