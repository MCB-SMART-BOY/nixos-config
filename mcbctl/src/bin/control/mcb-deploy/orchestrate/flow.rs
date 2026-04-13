use super::super::*;

impl App {
    pub(crate) fn deploy_flow(&mut self) -> Result<()> {
        self.banner();
        self.set_deploy_mode_prompt()?;
        self.validate_mode_conflicts()?;
        self.prompt_overwrite_mode()?;
        self.prompt_rebuild_upgrade()?;
        self.prompt_source_strategy()?;

        if !self.source_ref.is_empty() && self.allow_remote_head {
            self.warn("检测到来源策略冲突，将优先使用固定版本。");
            self.allow_remote_head = false;
        }

        self.section("环境检查");
        self.check_env()?;
        self.progress_step("环境检查");

        let tmp_dir = create_temp_dir("mcbctl-source")?;
        self.tmp_dir = Some(tmp_dir.clone());

        let result = (|| -> Result<()> {
            self.section("准备源代码");
            loop {
                if self.prepare_source_repo(&tmp_dir).is_ok() {
                    break;
                }
                if !self.is_tty() {
                    bail!("仓库拉取失败，请检查网络或更换来源策略");
                }
                let pick = self.menu_prompt(
                    "准备源代码失败，下一步",
                    1,
                    &[
                        "重试当前来源".to_string(),
                        "重新选择来源策略".to_string(),
                        "退出".to_string(),
                    ],
                )?;
                match pick {
                    1 => continue,
                    2 => {
                        self.source_choice_set = false;
                        self.prompt_source_strategy()?;
                    }
                    3 => bail!("已退出"),
                    _ => {}
                }
            }
            self.progress_step("准备源代码");

            self.section("仓库自检");
            self.self_check_repo(&tmp_dir)?;
            self.progress_step("仓库自检");

            self.wizard_flow(&tmp_dir)?;
            if self.deploy_mode == DeployMode::UpdateExisting {
                self.preserve_existing_local_override(&tmp_dir)?;
            } else {
                self.ensure_host_entry(&tmp_dir)?;
                self.ensure_user_home_entries(&tmp_dir)?;
                if !self.created_home_users.is_empty() {
                    self.warn(&format!(
                        "已自动创建用户 Home Manager 模板：{}",
                        self.created_home_users.join(" ")
                    ));
                }
                self.write_local_override(&tmp_dir)?;
            }
            self.ensure_target_hardware_config()?;
            self.progress_step("收集配置");
            self.confirm_continue("确认以上配置并继续同步？")?;

            self.section("同步与构建");
            self.prepare_etc_dir()?;
            self.progress_step("准备覆盖策略");

            self.sync_repo_to_etc(&tmp_dir)?;
            self.progress_step("同步配置");
            self.confirm_continue("配置已同步，继续重建系统？")?;
            if !self.rebuild_system()? {
                if !self.dns_enabled {
                    self.log("尝试临时切换阿里云 DNS 后重试重建");
                    if !self.temp_dns_enable()? {
                        self.warn("临时 DNS 设置失败，将继续使用当前 DNS 重试重建。");
                    }
                    if !self.rebuild_system()? {
                        bail!("系统重建失败，请检查日志");
                    }
                } else {
                    bail!("系统重建失败，请检查日志");
                }
            }
            self.progress_step("系统重建");
            Ok(())
        })();

        self.temp_dns_disable();
        if let Some(tmp) = self.tmp_dir.take() {
            fs::remove_dir_all(tmp).ok();
        }
        result
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        match self.run_action {
            RunAction::Deploy => self.deploy_flow(),
            RunAction::Release => self.release_flow(),
        }
    }
}
