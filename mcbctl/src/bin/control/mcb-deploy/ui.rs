use super::*;

#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::collections::VecDeque;

#[cfg(test)]
thread_local! {
    static TEST_TTY_OVERRIDE: RefCell<Option<bool>> = const { RefCell::new(None) };
    static TEST_PROMPT_INPUTS: RefCell<VecDeque<String>> = const { RefCell::new(VecDeque::new()) };
    static TEST_OUTPUT: RefCell<String> = const { RefCell::new(String::new()) };
}

#[cfg(test)]
pub(crate) struct TestUiGuard;

#[cfg(test)]
impl Drop for TestUiGuard {
    fn drop(&mut self) {
        TEST_TTY_OVERRIDE.with(|value| *value.borrow_mut() = None);
        TEST_PROMPT_INPUTS.with(|inputs| inputs.borrow_mut().clear());
        TEST_OUTPUT.with(|output| output.borrow_mut().clear());
    }
}

impl App {
    #[cfg(test)]
    pub(crate) fn install_test_ui(force_tty: bool, inputs: &[&str]) -> TestUiGuard {
        TEST_TTY_OVERRIDE.with(|value| *value.borrow_mut() = Some(force_tty));
        TEST_PROMPT_INPUTS.with(|queue| {
            let mut queue = queue.borrow_mut();
            queue.clear();
            queue.extend(inputs.iter().map(|input| input.to_string()));
        });
        TEST_OUTPUT.with(|output| output.borrow_mut().clear());
        TestUiGuard
    }

    #[cfg(test)]
    pub(crate) fn take_test_output() -> String {
        TEST_OUTPUT.with(|output| std::mem::take(&mut *output.borrow_mut()))
    }

    fn write_raw(&self, text: &str) {
        #[cfg(test)]
        {
            TEST_OUTPUT.with(|output| output.borrow_mut().push_str(text));
        }
        #[cfg(not(test))]
        {
            print!("{text}");
        }
    }

    fn write_line(&self, text: &str) {
        #[cfg(test)]
        {
            TEST_OUTPUT.with(|output| {
                let mut output = output.borrow_mut();
                output.push_str(text);
                output.push('\n');
            });
        }
        #[cfg(not(test))]
        {
            println!("{text}");
        }
    }

    pub(crate) fn msg(&self, level: &str, text: &str) {
        self.write_line(&format!("[{}] {}", level, text));
    }

    pub(crate) fn log(&self, text: &str) {
        self.msg("信息", text);
    }

    pub(crate) fn success(&self, text: &str) {
        self.msg("完成", text);
    }

    pub(crate) fn warn(&self, text: &str) {
        self.msg("警告", text);
    }

    pub(crate) fn section(&self, text: &str) {
        self.write_line(&format!("\n{text}"));
    }

    pub(crate) fn note(&self, text: &str) {
        self.write_line(text);
    }

    pub(crate) fn banner(&self) {
        self.write_line("==========================================");
        self.write_line("  NixOS 一键部署（mcbctl）");
        self.write_line("==========================================");
    }

    pub(crate) fn is_tty(&self) -> bool {
        #[cfg(test)]
        if let Some(tty) = TEST_TTY_OVERRIDE.with(|value| *value.borrow()) {
            return tty;
        }
        io::stdin().is_terminal() && io::stdout().is_terminal()
    }

    pub(crate) fn progress_step(&mut self, label: &str) {
        self.progress_current = self.progress_current.saturating_add(1);
        let width = 24u32;
        let filled = (self.progress_current * width) / self.progress_total.max(1);
        let empty = width.saturating_sub(filled);
        let bar = format!(
            "{}{}",
            "#".repeat(filled as usize),
            "-".repeat(empty as usize)
        );
        self.write_line(&format!(
            "进度: [{}] {}/{} {}",
            bar, self.progress_current, self.progress_total, label
        ));
    }

    pub(crate) fn usage(&self) {
        self.write_line(
            "用法:
  mcb-deploy
  mcb-deploy release

说明:
  默认模式为全交互部署向导，不需要任何命令行参数。
  所有配置项（部署模式、来源、覆盖策略、用户、权限、GPU、TUN 等）
  均在执行过程中通过菜单选择。

  release 模式用于发布新版本：创建 tag、发布 GitHub Release，
  并触发跨平台预编译产物上传流程。",
        );
    }

    pub(crate) fn prompt_line(&self, prompt: &str) -> Result<String> {
        #[cfg(test)]
        if let Some(input) = TEST_PROMPT_INPUTS.with(|queue| queue.borrow_mut().pop_front()) {
            self.write_raw(prompt);
            return Ok(input);
        }
        self.write_raw(prompt);
        io::stdout().flush().context("刷新输出失败")?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).context("读取输入失败")?;
        Ok(input)
    }

    pub(crate) fn menu_prompt(
        &self,
        title: &str,
        default_index: usize,
        options: &[String],
    ) -> Result<usize> {
        if options.is_empty() {
            bail!("菜单选项不能为空");
        }
        loop {
            self.write_line(&format!("\n{title}"));
            for (idx, opt) in options.iter().enumerate() {
                self.write_line(&format!("  {}) {}", idx + 1, opt));
            }
            self.write_raw(&format!(
                "请选择 [1-{}]（默认 {}，输入 q 退出）： ",
                options.len(),
                default_index
            ));
            let input = self.prompt_line("")?;
            let input = input.trim();
            if input.eq_ignore_ascii_case("q") {
                bail!("已退出");
            }
            if input.is_empty() {
                return Ok(default_index);
            }
            if let Ok(v) = input.parse::<usize>()
                && v >= 1
                && v <= options.len()
            {
                return Ok(v);
            }
            self.write_line("无效选择，请重试。");
        }
    }

    pub(crate) fn ask_bool(&self, prompt: &str, default: bool) -> Result<bool> {
        if !self.is_tty() {
            return Ok(default);
        }
        let default_idx = if default { 1 } else { 2 };
        let pick = self.menu_prompt(
            prompt,
            default_idx,
            &["是 (true)".to_string(), "否 (false)".to_string()],
        )?;
        Ok(pick == 1)
    }

    pub(crate) fn wizard_back_or_quit(&self, prompt: &str) -> Result<WizardAction> {
        let answer = self.prompt_line(&format!("{prompt} [c继续/b返回/q退出]（默认 c）： "))?;
        let a = answer.trim();
        if a.eq_ignore_ascii_case("b") {
            Ok(WizardAction::Back)
        } else if a.eq_ignore_ascii_case("q") {
            bail!("已退出")
        } else {
            Ok(WizardAction::Continue)
        }
    }

    pub(crate) fn confirm_continue(&self, prompt: &str) -> Result<()> {
        if !self.is_tty() {
            return Ok(());
        }
        let answer = self.prompt_line(&format!("{prompt} [Y/n] "))?;
        if answer.trim().eq_ignore_ascii_case("n") {
            bail!("已退出");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn wizard_back_or_quit_maps_empty_back_and_quit_inputs() -> Result<()> {
        let app = test_app();

        let _ui = App::install_test_ui(true, &[""]);
        assert_eq!(
            app.wizard_back_or_quit("确认以上配置")?,
            WizardAction::Continue
        );
        drop(_ui);

        let _ui = App::install_test_ui(true, &["b"]);
        assert_eq!(app.wizard_back_or_quit("确认以上配置")?, WizardAction::Back);
        drop(_ui);

        let _ui = App::install_test_ui(true, &["q"]);
        let err = app
            .wizard_back_or_quit("确认以上配置")
            .expect_err("q should exit");
        assert!(err.to_string().contains("已退出"));
        Ok(())
    }

    #[test]
    fn confirm_continue_respects_tty_confirmation_input() -> Result<()> {
        let app = test_app();

        let _ui = App::install_test_ui(true, &[""]);
        app.confirm_continue("继续？")?;
        drop(_ui);

        let _ui = App::install_test_ui(true, &["n"]);
        let err = app.confirm_continue("继续？").expect_err("n should abort");
        assert!(err.to_string().contains("已退出"));
        Ok(())
    }

    fn test_app() -> App {
        App {
            repo_dir: PathBuf::from("/tmp/repo"),
            source_dir_override: None,
            repo_urls: Vec::new(),
            branch: "main".to_string(),
            source_ref: String::new(),
            allow_remote_head: false,
            source_commit: String::new(),
            source_choice_set: false,
            target_name: String::new(),
            target_users: Vec::new(),
            target_admin_users: Vec::new(),
            deploy_mode: DeployMode::ManageUsers,
            deploy_mode_set: false,
            force_remote_source: false,
            overwrite_mode: OverwriteMode::Ask,
            overwrite_mode_set: false,
            per_user_tun_enabled: false,
            host_profile_kind: HostProfileKind::Unknown,
            user_tun: BTreeMap::new(),
            user_dns: BTreeMap::new(),
            server_overrides_enabled: false,
            server_enable_network_cli: String::new(),
            server_enable_network_gui: String::new(),
            server_enable_shell_tools: String::new(),
            server_enable_wayland_tools: String::new(),
            server_enable_system_tools: String::new(),
            server_enable_geek_tools: String::new(),
            server_enable_gaming: String::new(),
            server_enable_insecure_tools: String::new(),
            server_enable_docker: String::new(),
            server_enable_libvirtd: String::new(),
            created_home_users: Vec::new(),
            gpu_override: false,
            gpu_override_from_detection: false,
            gpu_mode: String::new(),
            gpu_igpu_vendor: String::new(),
            gpu_prime_mode: String::new(),
            gpu_intel_bus: String::new(),
            gpu_amd_bus: String::new(),
            gpu_nvidia_bus: String::new(),
            gpu_nvidia_open: String::new(),
            gpu_specialisations_enabled: false,
            gpu_specialisations_set: false,
            gpu_specialisation_modes: Vec::new(),
            detected_gpu: DetectedGpuProfile::default(),
            mode: "switch".to_string(),
            rebuild_upgrade: false,
            rebuild_upgrade_set: false,
            etc_dir: PathBuf::from("/tmp/etc-nixos"),
            dns_enabled: false,
            temp_dns_backend: String::new(),
            temp_dns_backup: None,
            temp_dns_iface: String::new(),
            tmp_dir: None,
            sudo_cmd: None,
            rootless: false,
            run_action: RunAction::Deploy,
            progress_total: 7,
            progress_current: 0,
            git_clone_timeout_sec: 90,
        }
    }
}
