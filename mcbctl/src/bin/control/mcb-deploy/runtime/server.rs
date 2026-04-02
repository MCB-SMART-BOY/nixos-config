use super::super::*;

impl App {
    pub(crate) fn reset_server_overrides(&mut self) {
        self.server_overrides_enabled = false;
        self.server_enable_network_cli.clear();
        self.server_enable_network_gui.clear();
        self.server_enable_shell_tools.clear();
        self.server_enable_wayland_tools.clear();
        self.server_enable_system_tools.clear();
        self.server_enable_geek_tools.clear();
        self.server_enable_gaming.clear();
        self.server_enable_insecure_tools.clear();
        self.server_enable_docker.clear();
        self.server_enable_libvirtd.clear();
    }

    pub(crate) fn configure_server_overrides(&mut self) -> Result<WizardAction> {
        if !self.is_tty() {
            self.reset_server_overrides();
            return Ok(WizardAction::Continue);
        }

        let pick = self.menu_prompt(
            "服务器软件覆盖",
            2,
            &[
                "启用服务器包组覆盖".to_string(),
                "沿用主机现有配置".to_string(),
                "返回".to_string(),
            ],
        )?;

        match pick {
            1 => self.server_overrides_enabled = true,
            2 => {
                self.reset_server_overrides();
                return Ok(WizardAction::Continue);
            }
            3 => return Ok(WizardAction::Back),
            _ => {}
        }

        let ask = |app: &App, name: &str, default: bool| -> Result<String> {
            Ok(if app.ask_bool(&format!("{name}？"), default)? {
                "true".to_string()
            } else {
                "false".to_string()
            })
        };

        self.server_enable_network_cli = ask(self, "启用网络 CLI 包", true)?;
        self.server_enable_network_gui = ask(self, "启用网络 GUI 包", false)?;
        self.server_enable_shell_tools = ask(self, "启用 Shell 工具", true)?;
        self.server_enable_wayland_tools = ask(self, "启用 Wayland 工具", false)?;
        self.server_enable_system_tools = ask(self, "启用系统工具", true)?;
        self.server_enable_geek_tools = ask(self, "启用 Geek 工具", true)?;
        self.server_enable_gaming = ask(self, "启用游戏工具", false)?;
        self.server_enable_insecure_tools = ask(self, "启用不安全工具", false)?;
        self.server_enable_docker = ask(self, "启用 Docker", true)?;
        self.server_enable_libvirtd = ask(self, "启用 Libvirtd", false)?;

        Ok(WizardAction::Continue)
    }
}
