use super::super::*;

fn summarize_cleanup_failures(context: &str, failures: &[String]) -> String {
    if failures.is_empty() {
        return context.to_string();
    }

    format!("{context}: {}", failures.join(" | "))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourcePrepareFailureAction {
    Retry,
    ReselectSource,
    Exit,
}

trait DeployFlowRunner {
    fn check_env(&mut self, app: &mut App) -> Result<()>;
    fn prompt_source_strategy(&mut self, app: &mut App) -> Result<()>;
    fn create_temp_dir(&mut self, prefix: &str) -> Result<PathBuf>;
    fn prepare_source_repo(&mut self, app: &mut App, tmp_dir: &Path) -> Result<()>;
    fn handle_source_prepare_failure(
        &mut self,
        app: &mut App,
        detail: &str,
    ) -> Result<SourcePrepareFailureAction>;
    fn self_check_repo(&mut self, app: &App, repo_dir: &Path) -> Result<()>;
    fn collect_deploy_config(&mut self, app: &mut App, repo_dir: &Path) -> Result<()>;
    fn prepare_etc_dir(&mut self, app: &mut App) -> Result<()>;
    fn sync_repo_to_etc(&mut self, app: &App, repo_dir: &Path) -> Result<()>;
    fn rebuild_system(&mut self, app: &App) -> Result<bool>;
    fn temp_dns_enable(&mut self, app: &mut App) -> Result<bool>;
    fn temp_dns_disable(&mut self, app: &mut App) -> Result<()>;
    fn remove_dir_all(&mut self, path: &Path) -> Result<()>;
}

struct RealDeployFlowRunner;

impl DeployFlowRunner for RealDeployFlowRunner {
    fn check_env(&mut self, app: &mut App) -> Result<()> {
        app.check_env()
    }

    fn prompt_source_strategy(&mut self, app: &mut App) -> Result<()> {
        app.prompt_source_strategy()
    }

    fn create_temp_dir(&mut self, prefix: &str) -> Result<PathBuf> {
        create_temp_dir(prefix)
    }

    fn prepare_source_repo(&mut self, app: &mut App, tmp_dir: &Path) -> Result<()> {
        app.prepare_source_repo(tmp_dir)
    }

    fn handle_source_prepare_failure(
        &mut self,
        app: &mut App,
        detail: &str,
    ) -> Result<SourcePrepareFailureAction> {
        app.warn(&format!("准备源代码失败：{detail}"));
        if !app.is_tty() {
            bail!("准备源代码失败：{detail}");
        }
        let pick = app.menu_prompt(
            "准备源代码失败，下一步",
            1,
            &[
                "重试当前来源".to_string(),
                "重新选择来源策略".to_string(),
                "退出".to_string(),
            ],
        )?;
        Ok(match pick {
            1 => SourcePrepareFailureAction::Retry,
            2 => SourcePrepareFailureAction::ReselectSource,
            3 => SourcePrepareFailureAction::Exit,
            _ => SourcePrepareFailureAction::Retry,
        })
    }

    fn self_check_repo(&mut self, app: &App, repo_dir: &Path) -> Result<()> {
        app.self_check_repo(repo_dir)
    }

    fn collect_deploy_config(&mut self, app: &mut App, repo_dir: &Path) -> Result<()> {
        app.wizard_flow(repo_dir)?;
        if app.deploy_mode == DeployMode::UpdateExisting {
            app.preserve_existing_local_override(repo_dir)?;
        } else {
            app.ensure_host_entry(repo_dir)?;
            app.ensure_user_home_entries(repo_dir)?;
            if !app.created_home_users.is_empty() {
                app.warn(&format!(
                    "已自动创建用户 Home Manager 模板：{}",
                    app.created_home_users.join(" ")
                ));
            }
            app.write_local_override(repo_dir)?;
        }
        app.ensure_target_hardware_config()
    }

    fn prepare_etc_dir(&mut self, app: &mut App) -> Result<()> {
        app.prepare_etc_dir()
    }

    fn sync_repo_to_etc(&mut self, app: &App, repo_dir: &Path) -> Result<()> {
        app.sync_repo_to_etc(repo_dir)
    }

    fn rebuild_system(&mut self, app: &App) -> Result<bool> {
        app.rebuild_system()
    }

    fn temp_dns_enable(&mut self, app: &mut App) -> Result<bool> {
        app.temp_dns_enable()
    }

    fn temp_dns_disable(&mut self, app: &mut App) -> Result<()> {
        app.temp_dns_disable()
    }

    fn remove_dir_all(&mut self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))
    }
}

fn deploy_flow_with_runner<R>(app: &mut App, runner: &mut R) -> Result<()>
where
    R: DeployFlowRunner,
{
    app.banner();
    app.set_deploy_mode_prompt()?;
    app.validate_mode_conflicts()?;
    app.prompt_overwrite_mode()?;
    app.prompt_rebuild_upgrade()?;
    runner.prompt_source_strategy(app)?;

    if !app.source_ref.is_empty() && app.allow_remote_head {
        app.warn("检测到来源策略冲突，将优先使用固定版本。");
        app.allow_remote_head = false;
    }

    app.section("环境检查");
    runner.check_env(app)?;
    app.progress_step("环境检查");

    let tmp_dir = runner.create_temp_dir("mcbctl-source")?;
    app.tmp_dir = Some(tmp_dir.clone());

    let result = (|| -> Result<()> {
        app.section("准备源代码");
        loop {
            match runner.prepare_source_repo(app, &tmp_dir) {
                Ok(()) => break,
                Err(err) => {
                    let detail = err.to_string();
                    match runner.handle_source_prepare_failure(app, &detail)? {
                        SourcePrepareFailureAction::Retry => continue,
                        SourcePrepareFailureAction::ReselectSource => {
                            app.source_choice_set = false;
                            runner.prompt_source_strategy(app)?;
                        }
                        SourcePrepareFailureAction::Exit => bail!("已退出"),
                    }
                }
            }
        }
        app.progress_step("准备源代码");

        app.section("仓库自检");
        runner.self_check_repo(app, &tmp_dir)?;
        app.progress_step("仓库自检");

        runner.collect_deploy_config(app, &tmp_dir)?;
        app.progress_step("收集配置");
        app.confirm_continue("确认以上配置并继续同步？")?;

        app.section("同步与构建");
        runner.prepare_etc_dir(app)?;
        app.progress_step("准备覆盖策略");

        runner.sync_repo_to_etc(app, &tmp_dir)?;
        app.progress_step("同步配置");
        app.confirm_continue("配置已同步，继续重建系统？")?;
        if !runner.rebuild_system(app)? {
            if !app.dns_enabled {
                app.log("尝试临时切换阿里云 DNS 后重试重建");
                if !runner.temp_dns_enable(app)? {
                    app.warn("临时 DNS 设置失败，将继续使用当前 DNS 重试重建。");
                }
                if !runner.rebuild_system(app)? {
                    bail!("系统重建失败，请检查日志");
                }
            } else {
                bail!("系统重建失败，请检查日志");
            }
        }
        app.progress_step("系统重建");
        Ok(())
    })();

    let mut cleanup_failures = Vec::new();
    if let Err(err) = runner.temp_dns_disable(app) {
        cleanup_failures.push(err.to_string());
    }
    if let Some(tmp) = app.tmp_dir.take()
        && let Err(err) = runner.remove_dir_all(&tmp)
    {
        cleanup_failures.push(format!("清理临时目录 {} 失败: {err}", tmp.display()));
    }

    if let Err(err) = result {
        for failure in cleanup_failures {
            app.warn(&failure);
        }
        return Err(err);
    }
    if !cleanup_failures.is_empty() {
        bail!(
            "{}",
            summarize_cleanup_failures("部署收尾清理失败", &cleanup_failures)
        );
    }
    Ok(())
}

impl App {
    pub(crate) fn deploy_flow(&mut self) -> Result<()> {
        let mut runner = RealDeployFlowRunner;
        deploy_flow_with_runner(self, &mut runner)
    }

    pub(crate) fn run(&mut self) -> Result<()> {
        match self.run_action {
            RunAction::Deploy => self.deploy_flow(),
            RunAction::Release => self.release_flow(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, VecDeque};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    struct TestFlowRunner {
        tmp_dir: PathBuf,
        calls: Vec<&'static str>,
        check_env_result: Option<Result<()>>,
        prompt_source_strategy_results: VecDeque<Result<Option<(bool, bool, String)>>>,
        prepare_source_results: VecDeque<Result<()>>,
        source_failure_actions: VecDeque<Result<SourcePrepareFailureAction>>,
        self_check_result: Option<Result<()>>,
        collect_config_result: Option<Result<()>>,
        prepare_etc_result: Option<Result<()>>,
        sync_result: Option<Result<()>>,
        rebuild_results: VecDeque<Result<bool>>,
        dns_enable_result: Option<Result<bool>>,
        dns_disable_result: Option<Result<()>>,
        remove_tmp_result: Option<Result<()>>,
    }

    impl TestFlowRunner {
        fn new(tmp_dir: PathBuf) -> Self {
            Self {
                tmp_dir,
                calls: Vec::new(),
                check_env_result: Some(Ok(())),
                prompt_source_strategy_results: VecDeque::from([Ok(None)]),
                prepare_source_results: VecDeque::from([Ok(())]),
                source_failure_actions: VecDeque::new(),
                self_check_result: Some(Ok(())),
                collect_config_result: Some(Ok(())),
                prepare_etc_result: Some(Ok(())),
                sync_result: Some(Ok(())),
                rebuild_results: VecDeque::from([Ok(true)]),
                dns_enable_result: Some(Ok(false)),
                dns_disable_result: Some(Ok(())),
                remove_tmp_result: Some(Ok(())),
            }
        }
    }

    impl DeployFlowRunner for TestFlowRunner {
        fn check_env(&mut self, _app: &mut App) -> Result<()> {
            self.calls.push("check_env");
            self.check_env_result.take().unwrap_or_else(|| Ok(()))
        }

        fn prompt_source_strategy(&mut self, app: &mut App) -> Result<()> {
            self.calls.push("prompt_source_strategy");
            let next = self
                .prompt_source_strategy_results
                .pop_front()
                .unwrap_or_else(|| Ok(None))?;
            if let Some((force_remote_source, allow_remote_head, source_ref)) = next {
                app.force_remote_source = force_remote_source;
                app.allow_remote_head = allow_remote_head;
                app.source_ref = source_ref;
            }
            app.source_choice_set = true;
            Ok(())
        }

        fn create_temp_dir(&mut self, _prefix: &str) -> Result<PathBuf> {
            self.calls.push("create_temp_dir");
            Ok(self.tmp_dir.clone())
        }

        fn prepare_source_repo(&mut self, _app: &mut App, _tmp_dir: &Path) -> Result<()> {
            self.calls.push("prepare_source_repo");
            self.prepare_source_results
                .pop_front()
                .unwrap_or_else(|| Ok(()))
        }

        fn handle_source_prepare_failure(
            &mut self,
            _app: &mut App,
            _detail: &str,
        ) -> Result<SourcePrepareFailureAction> {
            self.calls.push("handle_source_prepare_failure");
            self.source_failure_actions
                .pop_front()
                .unwrap_or_else(|| Ok(SourcePrepareFailureAction::Retry))
        }

        fn self_check_repo(&mut self, _app: &App, _repo_dir: &Path) -> Result<()> {
            self.calls.push("self_check_repo");
            self.self_check_result.take().unwrap_or_else(|| Ok(()))
        }

        fn collect_deploy_config(&mut self, _app: &mut App, _repo_dir: &Path) -> Result<()> {
            self.calls.push("collect_deploy_config");
            self.collect_config_result.take().unwrap_or_else(|| Ok(()))
        }

        fn prepare_etc_dir(&mut self, _app: &mut App) -> Result<()> {
            self.calls.push("prepare_etc_dir");
            self.prepare_etc_result.take().unwrap_or_else(|| Ok(()))
        }

        fn sync_repo_to_etc(&mut self, _app: &App, _repo_dir: &Path) -> Result<()> {
            self.calls.push("sync_repo_to_etc");
            self.sync_result.take().unwrap_or_else(|| Ok(()))
        }

        fn rebuild_system(&mut self, _app: &App) -> Result<bool> {
            self.calls.push("rebuild_system");
            self.rebuild_results.pop_front().unwrap_or_else(|| Ok(true))
        }

        fn temp_dns_enable(&mut self, app: &mut App) -> Result<bool> {
            self.calls.push("temp_dns_enable");
            let enabled = self.dns_enable_result.take().unwrap_or_else(|| Ok(false))?;
            if enabled {
                app.dns_enabled = true;
            }
            Ok(enabled)
        }

        fn temp_dns_disable(&mut self, _app: &mut App) -> Result<()> {
            self.calls.push("temp_dns_disable");
            self.dns_disable_result.take().unwrap_or_else(|| Ok(()))
        }

        fn remove_dir_all(&mut self, _path: &Path) -> Result<()> {
            self.calls.push("remove_dir_all");
            self.remove_tmp_result.take().unwrap_or_else(|| Ok(()))
        }
    }

    #[test]
    fn summarize_cleanup_failures_joins_messages() {
        let summary = summarize_cleanup_failures(
            "部署收尾清理失败",
            &["恢复 DNS 失败".to_string(), "清理临时目录失败".to_string()],
        );

        assert!(summary.contains("部署收尾清理失败"));
        assert!(summary.contains("恢复 DNS 失败"));
        assert!(summary.contains("清理临时目录失败"));
    }

    #[test]
    fn deploy_flow_stops_before_sync_when_prepare_etc_fails() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-prepare-etc")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir.clone());
        runner.prepare_etc_result = Some(Err(anyhow::anyhow!("prepare etc failed")));

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("prepare_etc_dir failure should stop deploy flow");

        assert!(err.to_string().contains("prepare etc failed"));
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_stops_before_rebuild_when_sync_fails() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-sync")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir.clone());
        runner.sync_result = Some(Err(anyhow::anyhow!("sync failed")));

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("sync failure should stop deploy flow");

        assert!(err.to_string().contains("sync failed"));
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "sync_repo_to_etc",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_retries_rebuild_after_dns_fallback() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-rebuild")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.rebuild_results = VecDeque::from([Ok(false), Ok(true)]);
        runner.dns_enable_result = Some(Ok(true));

        deploy_flow_with_runner(&mut app, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "sync_repo_to_etc",
                "rebuild_system",
                "temp_dns_enable",
                "rebuild_system",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_preserves_primary_error_when_cleanup_also_fails() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-cleanup-primary")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.sync_result = Some(Err(anyhow::anyhow!("sync failed")));
        runner.dns_disable_result = Some(Err(anyhow::anyhow!("dns disable failed")));
        runner.remove_tmp_result = Some(Err(anyhow::anyhow!("tmp cleanup failed")));

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("primary failure should be preserved");

        assert!(err.to_string().contains("sync failed"));
        assert!(!err.to_string().contains("dns disable failed"));
        assert!(!err.to_string().contains("tmp cleanup failed"));
        Ok(())
    }

    #[test]
    fn deploy_flow_fails_when_cleanup_fails_after_success() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-cleanup-success")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.remove_tmp_result = Some(Err(anyhow::anyhow!("tmp cleanup failed")));

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("cleanup failure after success should fail deploy flow");

        assert!(err.to_string().contains("部署收尾清理失败"));
        assert!(err.to_string().contains("tmp cleanup failed"));
        Ok(())
    }

    #[test]
    fn deploy_flow_tty_aborts_before_sync_when_user_declines_first_confirmation() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-confirm-sync-abort")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        let _ui = App::install_test_ui(true, &["2", "n"]);

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("first confirmation should abort deploy flow");

        assert!(err.to_string().contains("已退出"));
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_tty_aborts_before_rebuild_when_user_declines_second_confirmation() -> Result<()>
    {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-confirm-rebuild-abort")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        let _ui = App::install_test_ui(true, &["2", "", "n"]);

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("second confirmation should abort deploy flow");

        assert!(err.to_string().contains("已退出"));
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "sync_repo_to_etc",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_tty_cleanup_failure_after_confirmations_surfaces_aggregate_error() -> Result<()>
    {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-confirm-cleanup-fail")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.dns_disable_result = Some(Err(anyhow::anyhow!("dns disable failed")));
        runner.remove_tmp_result = Some(Err(anyhow::anyhow!("tmp cleanup failed")));
        let _ui = App::install_test_ui(true, &["2", "", ""]);

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("cleanup failure after confirmed success should fail deploy flow");
        let output = App::take_test_output();
        let cli_error = render_cli_error(&err);

        assert!(err.to_string().contains("部署收尾清理失败"));
        assert!(err.to_string().contains("dns disable failed"));
        assert!(err.to_string().contains("tmp cleanup failed"));
        assert!(output.contains("确认以上配置并继续同步？ [Y/n] "));
        assert!(output.contains("配置已同步，继续重建系统？ [Y/n] "));
        assert!(output.contains("同步与构建"));
        assert!(output.contains("进度: ["));
        assert!(output.contains("同步配置"));
        assert!(cli_error.contains("mcbctl: 部署收尾清理失败"));
        assert!(cli_error.contains("dns disable failed"));
        assert!(cli_error.contains("tmp cleanup failed"));
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "sync_repo_to_etc",
                "rebuild_system",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_retries_current_source_after_prepare_failure() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-retry")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.prepare_source_results =
            VecDeque::from([Err(anyhow::anyhow!("clone failed")), Ok(())]);
        runner.source_failure_actions = VecDeque::from([Ok(SourcePrepareFailureAction::Retry)]);

        deploy_flow_with_runner(&mut app, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "handle_source_prepare_failure",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "sync_repo_to_etc",
                "rebuild_system",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_reselects_source_strategy_after_prepare_failure() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-reselect")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.prompt_source_strategy_results = VecDeque::from([
            Ok(Some((true, false, "bad-pin".to_string()))),
            Ok(Some((true, true, String::new()))),
        ]);
        runner.prepare_source_results =
            VecDeque::from([Err(anyhow::anyhow!("checkout failed")), Ok(())]);
        runner.source_failure_actions =
            VecDeque::from([Ok(SourcePrepareFailureAction::ReselectSource)]);

        deploy_flow_with_runner(&mut app, &mut runner)?;

        assert!(app.force_remote_source);
        assert!(app.allow_remote_head);
        assert!(app.source_ref.is_empty());
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "handle_source_prepare_failure",
                "prompt_source_strategy",
                "prepare_source_repo",
                "self_check_repo",
                "collect_deploy_config",
                "prepare_etc_dir",
                "sync_repo_to_etc",
                "rebuild_system",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn deploy_flow_exits_after_prepare_failure_when_user_chooses_exit() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-exit")?;
        let mut app = test_app(tmp_dir.clone());
        let mut runner = TestFlowRunner::new(tmp_dir);
        runner.prepare_source_results = VecDeque::from([Err(anyhow::anyhow!("clone failed"))]);
        runner.source_failure_actions = VecDeque::from([Ok(SourcePrepareFailureAction::Exit)]);

        let err = deploy_flow_with_runner(&mut app, &mut runner)
            .expect_err("exit action should stop deploy flow");

        assert!(err.to_string().contains("已退出"));
        assert_eq!(
            runner.calls,
            vec![
                "prompt_source_strategy",
                "check_env",
                "create_temp_dir",
                "prepare_source_repo",
                "handle_source_prepare_failure",
                "temp_dns_disable",
                "remove_dir_all",
            ]
        );
        Ok(())
    }

    #[test]
    fn handle_source_prepare_failure_tty_maps_input_1_to_retry() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-menu-retry")?;
        let mut app = test_app(tmp_dir);
        let mut runner = RealDeployFlowRunner;
        let _ui = App::install_test_ui(true, &["1"]);

        let action = runner.handle_source_prepare_failure(&mut app, "clone failed")?;

        assert_eq!(action, SourcePrepareFailureAction::Retry);
        Ok(())
    }

    #[test]
    fn handle_source_prepare_failure_tty_maps_input_2_to_reselect_source() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-menu-reselect")?;
        let mut app = test_app(tmp_dir);
        let mut runner = RealDeployFlowRunner;
        let _ui = App::install_test_ui(true, &["2"]);

        let action = runner.handle_source_prepare_failure(&mut app, "checkout failed")?;

        assert_eq!(action, SourcePrepareFailureAction::ReselectSource);
        Ok(())
    }

    #[test]
    fn handle_source_prepare_failure_tty_maps_input_3_to_exit() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-menu-exit")?;
        let mut app = test_app(tmp_dir);
        let mut runner = RealDeployFlowRunner;
        let _ui = App::install_test_ui(true, &["3"]);

        let action = runner.handle_source_prepare_failure(&mut app, "checkout failed")?;

        assert_eq!(action, SourcePrepareFailureAction::Exit);
        Ok(())
    }

    #[test]
    fn handle_source_prepare_failure_tty_emits_warning_and_next_step_menu() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-menu-transcript")?;
        let mut app = test_app(tmp_dir);
        let mut runner = RealDeployFlowRunner;
        let _ui = App::install_test_ui(true, &["2"]);

        let action = runner.handle_source_prepare_failure(&mut app, "checkout failed")?;

        let output = App::take_test_output();
        assert_eq!(action, SourcePrepareFailureAction::ReselectSource);
        assert!(output.contains("[警告] 准备源代码失败：checkout failed"));
        assert!(output.contains("准备源代码失败，下一步"));
        assert!(output.contains("重试当前来源"));
        assert!(output.contains("重新选择来源策略"));
        assert!(output.contains("退出"));
        assert!(output.contains("请选择 [1-3]（默认 1，输入 q 退出）： "));
        Ok(())
    }

    #[test]
    fn handle_source_prepare_failure_tty_propagates_q_as_exit_error() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-menu-quit")?;
        let mut app = test_app(tmp_dir);
        let mut runner = RealDeployFlowRunner;
        let _ui = App::install_test_ui(true, &["q"]);

        let err = runner
            .handle_source_prepare_failure(&mut app, "checkout failed")
            .expect_err("q should abort source failure prompt");

        assert!(err.to_string().contains("已退出"));
        Ok(())
    }

    #[test]
    fn handle_source_prepare_failure_non_tty_bails_immediately() -> Result<()> {
        let _guard = test_lock();
        let tmp_dir = create_temp_dir("mcbctl-flow-source-menu-non-tty")?;
        let mut app = test_app(tmp_dir);
        let mut runner = RealDeployFlowRunner;
        let _ui = App::install_test_ui(false, &[]);

        let err = runner
            .handle_source_prepare_failure(&mut app, "clone failed")
            .expect_err("non-tty source failure should fail immediately");

        assert!(err.to_string().contains("准备源代码失败：clone failed"));
        Ok(())
    }

    fn create_temp_dir(prefix: &str) -> Result<PathBuf> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn test_app(repo_dir: PathBuf) -> App {
        App {
            repo_dir,
            source_dir_override: None,
            repo_urls: Vec::new(),
            branch: "rust脚本分支".to_string(),
            source_ref: "deadbeef".to_string(),
            allow_remote_head: false,
            source_commit: String::new(),
            source_choice_set: true,
            target_name: "demo".to_string(),
            target_users: vec!["mcb".to_string()],
            target_admin_users: vec!["mcb".to_string()],
            deploy_mode: DeployMode::ManageUsers,
            deploy_mode_set: true,
            force_remote_source: true,
            overwrite_mode: OverwriteMode::Backup,
            overwrite_mode_set: true,
            per_user_tun_enabled: false,
            host_profile_kind: HostProfileKind::Desktop,
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

    fn test_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
