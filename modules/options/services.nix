# 系统服务相关选项：放置经典调度器等显式服务开关。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb.services = {
    enableCron = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the classic cron daemon (crond/crontab/systemCronJobs support).";
    };

    enableAtd = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the at daemon for one-shot/batch scheduled command execution.";
    };
  };
}
