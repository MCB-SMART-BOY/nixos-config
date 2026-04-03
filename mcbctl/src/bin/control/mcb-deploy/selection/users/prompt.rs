use super::*;

impl App {
    pub(crate) fn select_existing_users_menu(&mut self, users: &[String]) -> Result<bool> {
        loop {
            let mut options = Vec::new();
            for user in users {
                if self.target_users.iter().any(|u| u == user) {
                    options.push(format!("[x] {user}"));
                } else {
                    options.push(format!("[ ] {user}"));
                }
            }
            options.push("完成".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("勾选已有用户（可重复切换）", 1, &options)?;
            if pick >= 1 && pick <= users.len() {
                self.toggle_target_user(&users[pick - 1]);
                continue;
            }
            if pick == users.len() + 1 {
                return Ok(true);
            }
            return Ok(false);
        }
    }

    pub(crate) fn select_admin_users_menu(&mut self) -> Result<bool> {
        loop {
            let mut options = Vec::new();
            for user in &self.target_users {
                if self.target_admin_users.iter().any(|u| u == user) {
                    options.push(format!("[x] {user}"));
                } else {
                    options.push(format!("[ ] {user}"));
                }
            }
            options.push("完成".to_string());
            options.push("返回".to_string());
            let pick = self.menu_prompt("勾选管理员用户（可重复切换）", 1, &options)?;
            if pick >= 1 && pick <= self.target_users.len() {
                let user = self.target_users[pick - 1].clone();
                self.toggle_admin_user(&user);
                continue;
            }
            if pick == self.target_users.len() + 1 {
                return Ok(true);
            }
            return Ok(false);
        }
    }

    pub(crate) fn prompt_users(&mut self, repo_dir: &Path) -> Result<WizardAction> {
        let default_user = self.resolve_default_user();
        if !self.is_tty() {
            if self.target_users.is_empty() {
                self.target_users = vec![default_user];
            }
            return Ok(WizardAction::Continue);
        }
        if self.target_users.is_empty() {
            self.target_users = vec![default_user.clone()];
        }

        loop {
            let current = if self.target_users.is_empty() {
                "未选择".to_string()
            } else {
                self.target_users.join(" ")
            };
            let pick = self.menu_prompt(
                &format!("选择用户（当前：{current}）"),
                1,
                &[
                    format!("仅使用默认用户 ({default_user})"),
                    "从已有 Home 用户中选择".to_string(),
                    "新增用户（手写用户名）".to_string(),
                    "清空已选用户".to_string(),
                    "完成".to_string(),
                    "返回".to_string(),
                    "退出".to_string(),
                ],
            )?;
            match pick {
                1 => {
                    self.target_users = vec![default_user.clone()];
                }
                2 => {
                    let mut existing = self.list_existing_home_users(repo_dir);
                    existing.sort();
                    existing.dedup();
                    if existing.is_empty() {
                        self.warn("未发现可选的已有 Home 用户目录。");
                        continue;
                    }
                    let _ = self.select_existing_users_menu(&existing)?;
                }
                3 => {
                    print!("输入新增用户名（留空取消）： ");
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).ok();
                    let input = input.trim();
                    if input.is_empty() {
                        continue;
                    }
                    if !is_valid_username(input) {
                        self.warn(&format!("用户名不合法：{input}"));
                        continue;
                    }
                    self.add_target_user(input);
                }
                4 => {
                    self.target_users.clear();
                }
                5 => {
                    if self.target_users.is_empty() {
                        self.warn("请至少选择一个用户。");
                        continue;
                    }
                    return Ok(WizardAction::Continue);
                }
                6 => return Ok(WizardAction::Back),
                7 => bail!("已退出"),
                _ => {}
            }
        }
    }

    pub(crate) fn prompt_admin_users(&mut self) -> Result<WizardAction> {
        if self.target_users.is_empty() {
            bail!("用户列表为空，无法选择管理员。");
        }
        let default_admin = self.target_users[0].clone();
        if !self.is_tty() {
            if self.target_admin_users.is_empty() {
                self.target_admin_users = vec![default_admin];
            }
            return Ok(WizardAction::Continue);
        }
        if self.target_admin_users.is_empty() {
            self.target_admin_users = vec![default_admin.clone()];
        }

        loop {
            let current = if self.target_admin_users.is_empty() {
                "未选择".to_string()
            } else {
                self.target_admin_users.join(" ")
            };
            let pick = self.menu_prompt(
                &format!("管理员权限（wheel，当前：{current}）"),
                1,
                &[
                    format!("仅主用户 ({default_admin})"),
                    "所有用户".to_string(),
                    "自定义勾选管理员".to_string(),
                    "清空管理员".to_string(),
                    "完成".to_string(),
                    "返回".to_string(),
                    "退出".to_string(),
                ],
            )?;
            match pick {
                1 => self.target_admin_users = vec![default_admin.clone()],
                2 => self.target_admin_users = self.target_users.clone(),
                3 => {
                    let _ = self.select_admin_users_menu()?;
                }
                4 => self.target_admin_users.clear(),
                5 => {
                    if self.target_admin_users.is_empty() {
                        self.warn("至少需要一个管理员用户。");
                        continue;
                    }
                    return Ok(WizardAction::Continue);
                }
                6 => return Ok(WizardAction::Back),
                7 => bail!("已退出"),
                _ => {}
            }
        }
    }
}
