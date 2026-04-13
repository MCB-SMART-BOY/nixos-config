use super::*;

impl App {
    pub(crate) fn msg(&self, level: &str, text: &str) {
        println!("[{}] {}", level, text);
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
        println!("\n{text}");
    }

    pub(crate) fn note(&self, text: &str) {
        println!("{text}");
    }

    pub(crate) fn banner(&self) {
        println!("==========================================");
        println!("  NixOS 一键部署（mcbctl）");
        println!("==========================================");
    }

    pub(crate) fn is_tty(&self) -> bool {
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
        println!(
            "进度: [{}] {}/{} {}",
            bar, self.progress_current, self.progress_total, label
        );
    }

    pub(crate) fn usage(&self) {
        println!(
            "用法:
  mcb-deploy
  mcb-deploy release

说明:
  默认模式为全交互部署向导，不需要任何命令行参数。
  所有配置项（部署模式、来源、覆盖策略、用户、权限、GPU、TUN 等）
  均在执行过程中通过菜单选择。

  release 模式用于发布新版本：创建 tag、发布 GitHub Release，
  并触发跨平台预编译产物上传流程。"
        );
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
            println!("\n{title}");
            for (idx, opt) in options.iter().enumerate() {
                println!("  {}) {}", idx + 1, opt);
            }
            print!(
                "请选择 [1-{}]（默认 {}，输入 q 退出）： ",
                options.len(),
                default_index
            );
            io::stdout().flush().ok();
            let mut input = String::new();
            io::stdin().read_line(&mut input).context("读取输入失败")?;
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
            println!("无效选择，请重试。");
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
        print!("{prompt} [c继续/b返回/q退出]（默认 c）： ");
        io::stdout().flush().ok();
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).ok();
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
        print!("{prompt} [Y/n] ");
        io::stdout().flush().ok();
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).ok();
        if answer.trim().eq_ignore_ascii_case("n") {
            bail!("已退出");
        }
        Ok(())
    }
}
