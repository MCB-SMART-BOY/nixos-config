use super::*;

impl App {
    pub(crate) fn prompt_source_strategy(&mut self) -> Result<()> {
        if self.source_choice_set {
            return Ok(());
        }
        let local_repo = self.detect_local_repo_dir();
        if !self.is_tty() {
            if self.deploy_mode == DeployMode::UpdateExisting {
                self.force_remote_source = true;
                self.allow_remote_head = true;
                self.source_ref.clear();
            } else if local_repo.is_some() {
                self.force_remote_source = false;
                self.allow_remote_head = false;
                self.source_ref.clear();
            } else {
                self.force_remote_source = true;
                self.allow_remote_head = false;
            }
            self.source_choice_set = true;
            return Ok(());
        }

        let mut options = Vec::<String>::new();
        let mut default_index = 1usize;
        if let Some(local) = &local_repo {
            options.push(format!("使用本地仓库（推荐）: {}", local.display()));
        }
        options.push("使用网络仓库固定版本（输入 commit/tag）".to_string());
        options.push("使用网络仓库最新版本（HEAD）".to_string());
        if self.deploy_mode == DeployMode::UpdateExisting {
            default_index = options.len();
        }
        let pick = self.menu_prompt("选择配置来源", default_index, &options)?;

        if local_repo.is_some() && pick == 1 {
            self.force_remote_source = false;
            self.allow_remote_head = false;
            self.source_ref.clear();
        } else {
            let mut remote_pick = pick;
            if local_repo.is_some() {
                remote_pick = pick.saturating_sub(1);
            }
            match remote_pick {
                1 => {
                    self.force_remote_source = true;
                    self.allow_remote_head = false;
                    loop {
                        let line = self.prompt_line("请输入远端固定版本（commit/tag）： ")?;
                        let v = line.trim();
                        if !v.is_empty() {
                            self.source_ref = v.to_string();
                            break;
                        }
                        println!("版本不能为空，请重试。");
                    }
                }
                2 => {
                    self.force_remote_source = true;
                    self.allow_remote_head = true;
                    self.source_ref.clear();
                }
                _ => {}
            }
        }
        self.source_choice_set = true;
        Ok(())
    }

    pub(crate) fn validate_mode_conflicts(&self) -> Result<()> {
        if self.deploy_mode == DeployMode::UpdateExisting && !self.target_users.is_empty() {
            bail!("仅更新模式不允许修改用户列表；该模式会保留现有用户与权限。");
        }
        Ok(())
    }

    pub(crate) fn require_remote_source_pin(&self) -> Result<()> {
        if self.allow_remote_head {
            self.warn("当前将跟随远端分支最新提交（存在供应链风险）。");
            return Ok(());
        }
        if self.source_ref.is_empty() {
            bail!(
                "未检测到本地仓库，且未选择远端固定版本；请在向导中选择固定版本或明确选择远端最新版本。"
            );
        }
        Ok(())
    }
}
