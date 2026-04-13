use super::super::*;

impl App {
    pub(crate) fn reset_tun_maps(&mut self) {
        self.user_tun.clear();
        self.user_dns.clear();
    }

    pub(crate) fn configure_per_user_tun(&mut self) -> Result<WizardAction> {
        if !self.is_tty() {
            return Ok(WizardAction::Continue);
        }
        self.section("Per-user TUN 配置");
        self.note("检测到当前主机已启用 per-user TUN。");
        self.note("请为每个用户指定独立 TUN 名称与 DNS 端口。");
        'retry: loop {
            self.user_tun.clear();
            self.user_dns.clear();
            for (idx, user) in self.target_users.iter().enumerate() {
                let default_iface = format!("tun{}", idx + 1);
                print!("用户 {user} 的 TUN 接口（默认 {default_iface}）： ");
                io::stdout().flush().ok();
                let mut iface = String::new();
                io::stdin().read_line(&mut iface).ok();
                let iface = iface.trim();
                let iface = if iface.is_empty() {
                    &default_iface
                } else {
                    iface
                };
                self.user_tun.insert(user.clone(), iface.to_string());

                let default_dns = 1053u16 + (idx as u16);
                print!("用户 {user} 的 DNS 端口（默认 {default_dns}）： ");
                io::stdout().flush().ok();
                let mut dns = String::new();
                io::stdin().read_line(&mut dns).ok();
                let dns = dns.trim();
                let port = if dns.is_empty() {
                    default_dns
                } else if let Ok(v) = dns.parse::<u16>() {
                    v
                } else {
                    self.warn("端口无效，请重新输入这一轮。");
                    self.user_tun.clear();
                    self.user_dns.clear();
                    continue 'retry;
                };
                self.user_dns.insert(user.clone(), port);
            }
            self.note("Per-user TUN 配置预览：");
            for user in &self.target_users {
                let iface = self.user_tun.get(user).cloned().unwrap_or_default();
                let dns = self.user_dns.get(user).copied().unwrap_or_default();
                self.note(&format!("  - {user}: {iface}, DNS {dns}"));
            }
            match self.wizard_back_or_quit("确认 Per-user TUN 配置？")? {
                WizardAction::Back => return Ok(WizardAction::Back),
                WizardAction::Continue => return Ok(WizardAction::Continue),
            }
        }
    }
}
