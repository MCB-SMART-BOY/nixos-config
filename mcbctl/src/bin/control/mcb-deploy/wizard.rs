use super::*;

impl App {
    pub(super) fn wizard_flow(&mut self, repo_dir: &Path) -> Result<()> {
        let mut step = 1u8;

        if self.deploy_mode == DeployMode::UpdateExisting {
            loop {
                match step {
                    1 => {
                        self.select_host(repo_dir)?;
                        self.validate_host(repo_dir)?;
                        self.detect_host_profile_kind(repo_dir);
                        step = 2;
                    }
                    2 => {
                        self.print_summary();
                        if self.is_tty() {
                            match self.wizard_back_or_quit("确认仅更新当前配置并继续？")?
                            {
                                WizardAction::Back => {
                                    self.target_name.clear();
                                    step = 1;
                                }
                                WizardAction::Continue => return Ok(()),
                            }
                        } else {
                            return Ok(());
                        }
                    }
                    _ => return Ok(()),
                }
            }
        }

        loop {
            match step {
                1 => {
                    self.select_host(repo_dir)?;
                    self.validate_host(repo_dir)?;
                    if self.host_exists(repo_dir) {
                        self.detect_host_profile_kind(repo_dir);
                    }
                    step = 2;
                }
                2 => {
                    match self.prompt_users(repo_dir)? {
                        WizardAction::Back => {
                            self.target_users.clear();
                            self.reset_admin_users();
                            self.reset_tun_maps();
                            self.reset_gpu_override();
                            self.reset_server_overrides();
                            self.target_name.clear();
                            step = 1;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    self.dedupe_users();
                    self.validate_users()?;
                    self.reset_admin_users();
                    self.reset_tun_maps();
                    self.reset_gpu_override();
                    self.reset_server_overrides();
                    step = 3;
                }
                3 => {
                    match self.prompt_admin_users()? {
                        WizardAction::Back => {
                            self.reset_admin_users();
                            step = 2;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    self.dedupe_admin_users();
                    self.validate_admin_users()?;
                    step = 4;
                }
                4 => {
                    self.per_user_tun_enabled = self.detect_per_user_tun(repo_dir);
                    if self.per_user_tun_enabled {
                        match self.configure_per_user_tun()? {
                            WizardAction::Back => {
                                self.reset_tun_maps();
                                step = 3;
                                continue;
                            }
                            WizardAction::Continue => {}
                        }
                    } else {
                        self.reset_tun_maps();
                    }
                    step = 5;
                }
                5 => {
                    if self.host_profile_kind == HostProfileKind::Server {
                        self.reset_gpu_override();
                        step = 6;
                        continue;
                    }
                    match self.configure_gpu()? {
                        WizardAction::Back => {
                            self.reset_gpu_override();
                            step = 4;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    step = 6;
                }
                6 => {
                    if self.host_profile_kind != HostProfileKind::Server {
                        self.reset_server_overrides();
                        step = 7;
                        continue;
                    }
                    match self.configure_server_overrides()? {
                        WizardAction::Back => {
                            self.reset_server_overrides();
                            step = 5;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                    step = 7;
                }
                7 => {
                    self.print_summary();
                    if self.is_tty() {
                        match self.wizard_back_or_quit("确认以上配置")? {
                            WizardAction::Back => {
                                step = if self.host_profile_kind == HostProfileKind::Server {
                                    6
                                } else {
                                    5
                                };
                            }
                            WizardAction::Continue => return Ok(()),
                        }
                    } else {
                        return Ok(());
                    }
                }
                _ => return Ok(()),
            }
        }
    }
}
