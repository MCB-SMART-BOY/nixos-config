use super::*;

trait WizardFlowRunner {
    fn prepare_host(&mut self, app: &mut App, repo_dir: &Path) -> Result<()>;
    fn prompt_users(&mut self, app: &mut App, repo_dir: &Path) -> Result<WizardAction>;
    fn prompt_admin_users(&mut self, app: &mut App) -> Result<WizardAction>;
    fn per_user_tun_enabled(&mut self, app: &App, repo_dir: &Path) -> bool;
    fn configure_per_user_tun(&mut self, app: &mut App) -> Result<WizardAction>;
    fn configure_gpu(&mut self, app: &mut App) -> Result<WizardAction>;
    fn configure_server_overrides(&mut self, app: &mut App) -> Result<WizardAction>;
    fn confirm_summary(&mut self, app: &mut App, prompt: &str) -> Result<WizardAction>;
}

struct RealWizardFlowRunner;

impl WizardFlowRunner for RealWizardFlowRunner {
    fn prepare_host(&mut self, app: &mut App, repo_dir: &Path) -> Result<()> {
        app.select_host(repo_dir)?;
        app.validate_host(repo_dir)?;
        if app.host_exists(repo_dir) {
            app.detect_host_profile_kind(repo_dir);
        }
        app.detect_host_gpu_profile();
        Ok(())
    }

    fn prompt_users(&mut self, app: &mut App, repo_dir: &Path) -> Result<WizardAction> {
        app.prompt_users(repo_dir)
    }

    fn prompt_admin_users(&mut self, app: &mut App) -> Result<WizardAction> {
        app.prompt_admin_users()
    }

    fn per_user_tun_enabled(&mut self, app: &App, repo_dir: &Path) -> bool {
        app.detect_per_user_tun(repo_dir)
    }

    fn configure_per_user_tun(&mut self, app: &mut App) -> Result<WizardAction> {
        app.configure_per_user_tun()
    }

    fn configure_gpu(&mut self, app: &mut App) -> Result<WizardAction> {
        app.configure_gpu()
    }

    fn configure_server_overrides(&mut self, app: &mut App) -> Result<WizardAction> {
        app.configure_server_overrides()
    }

    fn confirm_summary(&mut self, app: &mut App, prompt: &str) -> Result<WizardAction> {
        app.print_summary();
        if app.is_tty() {
            app.wizard_back_or_quit(prompt)
        } else {
            Ok(WizardAction::Continue)
        }
    }
}

fn previous_runtime_step(app: &App) -> u8 {
    if app.per_user_tun_enabled { 4 } else { 3 }
}

fn reset_selected_host_state(app: &mut App) {
    app.target_name.clear();
    app.host_profile_kind = HostProfileKind::Unknown;
    app.per_user_tun_enabled = false;
    app.detected_gpu = DetectedGpuProfile::default();
}

fn wizard_flow_with_runner<R>(app: &mut App, repo_dir: &Path, runner: &mut R) -> Result<()>
where
    R: WizardFlowRunner,
{
    let mut step = 1u8;

    if app.deploy_mode == DeployMode::UpdateExisting {
        loop {
            match step {
                1 => {
                    runner.prepare_host(app, repo_dir)?;
                    step = 2;
                }
                2 => match runner.confirm_summary(app, "确认仅更新当前配置并继续？")? {
                    WizardAction::Back => {
                        reset_selected_host_state(app);
                        step = 1;
                    }
                    WizardAction::Continue => return Ok(()),
                },
                _ => return Ok(()),
            }
        }
    }

    loop {
        match step {
            1 => {
                runner.prepare_host(app, repo_dir)?;
                step = 2;
            }
            2 => {
                match runner.prompt_users(app, repo_dir)? {
                    WizardAction::Back => {
                        app.target_users.clear();
                        app.reset_admin_users();
                        app.reset_tun_maps();
                        app.reset_gpu_override();
                        app.reset_server_overrides();
                        reset_selected_host_state(app);
                        step = 1;
                        continue;
                    }
                    WizardAction::Continue => {}
                }
                app.dedupe_users();
                app.validate_users()?;
                app.reset_admin_users();
                app.reset_tun_maps();
                app.reset_gpu_override();
                app.reset_server_overrides();
                step = 3;
            }
            3 => {
                match runner.prompt_admin_users(app)? {
                    WizardAction::Back => {
                        app.reset_admin_users();
                        step = 2;
                        continue;
                    }
                    WizardAction::Continue => {}
                }
                app.dedupe_admin_users();
                app.validate_admin_users()?;
                step = 4;
            }
            4 => {
                app.per_user_tun_enabled = runner.per_user_tun_enabled(app, repo_dir);
                if app.per_user_tun_enabled {
                    match runner.configure_per_user_tun(app)? {
                        WizardAction::Back => {
                            app.reset_tun_maps();
                            step = 3;
                            continue;
                        }
                        WizardAction::Continue => {}
                    }
                } else {
                    app.reset_tun_maps();
                }
                step = 5;
            }
            5 => {
                if app.host_profile_kind == HostProfileKind::Server {
                    app.reset_gpu_override();
                    step = 6;
                    continue;
                }
                match runner.configure_gpu(app)? {
                    WizardAction::Back => {
                        app.reset_gpu_override();
                        step = previous_runtime_step(app);
                        continue;
                    }
                    WizardAction::Continue => {}
                }
                step = 6;
            }
            6 => {
                if app.host_profile_kind != HostProfileKind::Server {
                    app.reset_server_overrides();
                    step = 7;
                    continue;
                }
                match runner.configure_server_overrides(app)? {
                    WizardAction::Back => {
                        app.reset_server_overrides();
                        step = previous_runtime_step(app);
                        continue;
                    }
                    WizardAction::Continue => {}
                }
                step = 7;
            }
            7 => match runner.confirm_summary(app, "确认以上配置")? {
                WizardAction::Back => {
                    step = if app.host_profile_kind == HostProfileKind::Server {
                        6
                    } else {
                        5
                    };
                }
                WizardAction::Continue => return Ok(()),
            },
            _ => return Ok(()),
        }
    }
}

impl App {
    pub(super) fn wizard_flow(&mut self, repo_dir: &Path) -> Result<()> {
        let mut runner = RealWizardFlowRunner;
        wizard_flow_with_runner(self, repo_dir, &mut runner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, VecDeque};

    #[derive(Clone, Debug)]
    struct Snapshot {
        target_name: String,
        target_users: Vec<String>,
        target_admin_users: Vec<String>,
        per_user_tun_enabled: bool,
        user_tun: BTreeMap<String, String>,
        user_dns: BTreeMap<String, u16>,
        gpu_override: bool,
        server_overrides_enabled: bool,
        server_enable_network_cli: String,
        server_enable_docker: String,
        host_profile_kind: HostProfileKind,
    }

    impl Snapshot {
        fn capture(app: &App) -> Self {
            Self {
                target_name: app.target_name.clone(),
                target_users: app.target_users.clone(),
                target_admin_users: app.target_admin_users.clone(),
                per_user_tun_enabled: app.per_user_tun_enabled,
                user_tun: app.user_tun.clone(),
                user_dns: app.user_dns.clone(),
                gpu_override: app.gpu_override,
                server_overrides_enabled: app.server_overrides_enabled,
                server_enable_network_cli: app.server_enable_network_cli.clone(),
                server_enable_docker: app.server_enable_docker.clone(),
                host_profile_kind: app.host_profile_kind,
            }
        }
    }

    struct TestWizardRunner {
        calls: Vec<&'static str>,
        prepare_host_results: VecDeque<Result<(String, HostProfileKind)>>,
        prompt_users_results: VecDeque<Result<WizardAction>>,
        prompt_users_targets: VecDeque<Option<Vec<String>>>,
        prompt_admin_results: VecDeque<Result<WizardAction>>,
        per_user_tun_enabled_results: VecDeque<bool>,
        tun_results: VecDeque<Result<WizardAction>>,
        gpu_results: VecDeque<Result<WizardAction>>,
        server_results: VecDeque<Result<WizardAction>>,
        server_enabled_results: VecDeque<bool>,
        summary_results: VecDeque<Result<WizardAction>>,
        prepare_host_snapshots: Vec<Snapshot>,
        prompt_users_snapshots: Vec<Snapshot>,
        prompt_admin_snapshots: Vec<Snapshot>,
        tun_snapshots: Vec<Snapshot>,
        gpu_snapshots: Vec<Snapshot>,
        server_snapshots: Vec<Snapshot>,
        summary_snapshots: Vec<Snapshot>,
    }

    impl TestWizardRunner {
        fn new() -> Self {
            Self {
                calls: Vec::new(),
                prepare_host_results: VecDeque::from([Ok((
                    "demo".to_string(),
                    HostProfileKind::Desktop,
                ))]),
                prompt_users_results: VecDeque::from([Ok(WizardAction::Continue)]),
                prompt_users_targets: VecDeque::from([None]),
                prompt_admin_results: VecDeque::from([Ok(WizardAction::Continue)]),
                per_user_tun_enabled_results: VecDeque::from([false]),
                tun_results: VecDeque::new(),
                gpu_results: VecDeque::from([Ok(WizardAction::Continue)]),
                server_results: VecDeque::new(),
                server_enabled_results: VecDeque::from([true]),
                summary_results: VecDeque::from([Ok(WizardAction::Continue)]),
                prepare_host_snapshots: Vec::new(),
                prompt_users_snapshots: Vec::new(),
                prompt_admin_snapshots: Vec::new(),
                tun_snapshots: Vec::new(),
                gpu_snapshots: Vec::new(),
                server_snapshots: Vec::new(),
                summary_snapshots: Vec::new(),
            }
        }
    }

    impl WizardFlowRunner for TestWizardRunner {
        fn prepare_host(&mut self, app: &mut App, _repo_dir: &Path) -> Result<()> {
            self.calls.push("prepare_host");
            self.prepare_host_snapshots.push(Snapshot::capture(app));
            let (name, kind) = self
                .prepare_host_results
                .pop_front()
                .unwrap_or_else(|| Ok(("demo".to_string(), HostProfileKind::Desktop)))?;
            app.target_name = name;
            app.host_profile_kind = kind;
            Ok(())
        }

        fn prompt_users(&mut self, app: &mut App, _repo_dir: &Path) -> Result<WizardAction> {
            self.calls.push("prompt_users");
            self.prompt_users_snapshots.push(Snapshot::capture(app));
            let action = self
                .prompt_users_results
                .pop_front()
                .unwrap_or_else(|| Ok(WizardAction::Continue))?;
            let configured_users = self.prompt_users_targets.pop_front().unwrap_or(None);
            if action == WizardAction::Continue {
                if let Some(users) = configured_users {
                    app.target_users = users;
                } else if app.target_users.is_empty() {
                    app.target_users = vec!["mcb".to_string()];
                }
            }
            Ok(action)
        }

        fn prompt_admin_users(&mut self, app: &mut App) -> Result<WizardAction> {
            self.calls.push("prompt_admin_users");
            self.prompt_admin_snapshots.push(Snapshot::capture(app));
            let action = self
                .prompt_admin_results
                .pop_front()
                .unwrap_or_else(|| Ok(WizardAction::Continue))?;
            if action == WizardAction::Continue
                && app.target_admin_users.is_empty()
                && !app.target_users.is_empty()
            {
                app.target_admin_users = vec![app.target_users[0].clone()];
            }
            Ok(action)
        }

        fn per_user_tun_enabled(&mut self, app: &App, _repo_dir: &Path) -> bool {
            self.calls.push("per_user_tun_enabled");
            self.tun_snapshots.push(Snapshot::capture(app));
            self.per_user_tun_enabled_results
                .pop_front()
                .unwrap_or(false)
        }

        fn configure_per_user_tun(&mut self, app: &mut App) -> Result<WizardAction> {
            self.calls.push("configure_per_user_tun");
            let action = self
                .tun_results
                .pop_front()
                .unwrap_or_else(|| Ok(WizardAction::Continue))?;
            for (idx, user) in app.target_users.iter().enumerate() {
                app.user_tun.insert(user.clone(), format!("tun{}", idx + 1));
                app.user_dns.insert(user.clone(), 1053 + (idx as u16));
            }
            Ok(action)
        }

        fn configure_gpu(&mut self, app: &mut App) -> Result<WizardAction> {
            self.calls.push("configure_gpu");
            self.gpu_snapshots.push(Snapshot::capture(app));
            let action = self
                .gpu_results
                .pop_front()
                .unwrap_or_else(|| Ok(WizardAction::Continue))?;
            app.gpu_override = true;
            app.gpu_mode = "hybrid".to_string();
            Ok(action)
        }

        fn configure_server_overrides(&mut self, app: &mut App) -> Result<WizardAction> {
            self.calls.push("configure_server_overrides");
            self.server_snapshots.push(Snapshot::capture(app));
            let action = self
                .server_results
                .pop_front()
                .unwrap_or_else(|| Ok(WizardAction::Continue))?;
            if action == WizardAction::Continue {
                if self.server_enabled_results.pop_front().unwrap_or(true) {
                    app.server_overrides_enabled = true;
                    app.server_enable_network_cli = "true".to_string();
                    app.server_enable_docker = "true".to_string();
                } else {
                    app.reset_server_overrides();
                }
            }
            Ok(action)
        }

        fn confirm_summary(&mut self, app: &mut App, _prompt: &str) -> Result<WizardAction> {
            self.calls.push("confirm_summary");
            self.summary_snapshots.push(Snapshot::capture(app));
            self.summary_results
                .pop_front()
                .unwrap_or_else(|| Ok(WizardAction::Continue))
        }
    }

    #[test]
    fn update_existing_back_restarts_host_selection_with_cleared_target() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-update-back")?;
        let mut app = test_app(repo_dir.clone());
        app.deploy_mode = DeployMode::UpdateExisting;
        let mut runner = TestWizardRunner::new();
        runner.prepare_host_results = VecDeque::from([
            Ok(("demo".to_string(), HostProfileKind::Desktop)),
            Ok(("other".to_string(), HostProfileKind::Desktop)),
        ]);
        runner.summary_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "confirm_summary",
                "prepare_host",
                "confirm_summary"
            ]
        );
        assert_eq!(runner.prepare_host_snapshots.len(), 2);
        assert!(runner.prepare_host_snapshots[1].target_name.is_empty());
        assert_eq!(
            runner.prepare_host_snapshots[1].host_profile_kind,
            HostProfileKind::Unknown
        );
        assert!(!runner.prepare_host_snapshots[1].per_user_tun_enabled);
        assert_eq!(app.target_name, "other");
        Ok(())
    }

    #[test]
    fn users_back_restarts_host_selection_with_cleared_host_state() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-users-back")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prepare_host_results = VecDeque::from([
            Ok(("demo".to_string(), HostProfileKind::Server)),
            Ok(("other".to_string(), HostProfileKind::Desktop)),
        ]);
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_gpu",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.prepare_host_snapshots.len(), 2);
        assert!(runner.prepare_host_snapshots[1].target_name.is_empty());
        assert_eq!(
            runner.prepare_host_snapshots[1].host_profile_kind,
            HostProfileKind::Unknown
        );
        assert!(!runner.prepare_host_snapshots[1].per_user_tun_enabled);
        assert_eq!(app.target_name, "other");
        Ok(())
    }

    #[test]
    fn users_back_restarts_host_selection_with_cleared_user_and_runtime_state() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-users-back-clears-runtime")?;
        let mut app = test_app(repo_dir.clone());
        app.target_users = vec!["old-user".to_string()];
        app.target_admin_users = vec!["old-admin".to_string()];
        app.per_user_tun_enabled = true;
        app.user_tun
            .insert("old-user".to_string(), "tun9".to_string());
        app.user_dns.insert("old-user".to_string(), 1053);
        app.gpu_override = true;
        app.server_overrides_enabled = true;
        app.server_enable_network_cli = "true".to_string();
        app.server_enable_docker = "true".to_string();
        let mut runner = TestWizardRunner::new();
        runner.prepare_host_results = VecDeque::from([
            Ok(("demo".to_string(), HostProfileKind::Server)),
            Ok(("other".to_string(), HostProfileKind::Desktop)),
        ]);
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(runner.prepare_host_snapshots.len(), 2);
        let cleared = &runner.prepare_host_snapshots[1];
        assert!(cleared.target_name.is_empty());
        assert!(cleared.target_users.is_empty());
        assert!(cleared.target_admin_users.is_empty());
        assert!(!cleared.per_user_tun_enabled);
        assert!(cleared.user_tun.is_empty());
        assert!(cleared.user_dns.is_empty());
        assert!(!cleared.gpu_override);
        assert!(!cleared.server_overrides_enabled);
        assert!(cleared.server_enable_network_cli.is_empty());
        assert!(cleared.server_enable_docker.is_empty());
        assert_eq!(cleared.host_profile_kind, HostProfileKind::Unknown);
        Ok(())
    }

    #[test]
    fn admin_back_revisits_users_with_cleared_admins_and_new_primary_user() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-admin-back-clears-admins")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.prompt_users_targets = VecDeque::from([
            Some(vec!["alice".to_string(), "bob".to_string()]),
            Some(vec!["charlie".to_string()]),
        ]);
        runner.prompt_admin_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_gpu",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.prompt_admin_snapshots.len(), 2);
        assert_eq!(
            runner.prompt_admin_snapshots[0].target_users,
            vec!["alice".to_string(), "bob".to_string()]
        );
        assert!(
            runner.prompt_admin_snapshots[0]
                .target_admin_users
                .is_empty()
        );
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_users,
            vec!["charlie".to_string()]
        );
        assert!(
            runner.prompt_admin_snapshots[1]
                .target_admin_users
                .is_empty()
        );
        assert_eq!(app.target_users, vec!["charlie".to_string()]);
        assert_eq!(app.target_admin_users, vec!["charlie".to_string()]);
        Ok(())
    }

    #[test]
    fn tun_reentry_after_users_change_sees_cleared_maps_and_new_user_set() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-tun-users-change")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.prompt_users_targets = VecDeque::from([
            Some(vec!["alice".to_string(), "bob".to_string()]),
            Some(vec!["charlie".to_string()]),
        ]);
        runner.prompt_admin_results = VecDeque::from([
            Ok(WizardAction::Continue),
            Ok(WizardAction::Back),
            Ok(WizardAction::Continue),
        ]);
        runner.per_user_tun_enabled_results = VecDeque::from([true, true]);
        runner.tun_results = VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_per_user_tun",
                "prompt_admin_users",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_per_user_tun",
                "configure_gpu",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.tun_snapshots.len(), 2);
        assert_eq!(
            runner.tun_snapshots[0].target_users,
            vec!["alice".to_string(), "bob".to_string()]
        );
        assert_eq!(
            runner.tun_snapshots[1].target_users,
            vec!["charlie".to_string()]
        );
        assert!(runner.tun_snapshots[1].user_tun.is_empty());
        assert!(runner.tun_snapshots[1].user_dns.is_empty());
        assert_eq!(app.target_users, vec!["charlie".to_string()]);
        assert_eq!(app.user_tun.get("charlie"), Some(&"tun1".to_string()));
        assert_eq!(app.user_dns.get("charlie"), Some(&1053));
        assert_eq!(app.user_tun.len(), 1);
        assert_eq!(app.user_dns.len(), 1);
        Ok(())
    }

    #[test]
    fn gpu_back_without_tun_returns_to_admin_step_and_clears_gpu_state() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-gpu-back")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.prompt_admin_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.per_user_tun_enabled_results = VecDeque::from([false, false]);
        runner.gpu_results = VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_gpu",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_gpu",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.prompt_admin_snapshots.len(), 2);
        assert!(!runner.prompt_admin_snapshots[1].gpu_override);
        assert!(!runner.prompt_admin_snapshots[1].per_user_tun_enabled);
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_users,
            vec!["mcb".to_string()]
        );
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_admin_users,
            vec!["mcb".to_string()]
        );
        Ok(())
    }

    #[test]
    fn tun_back_returns_to_admin_step_and_clears_tun_maps() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-tun-back")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.prompt_admin_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.per_user_tun_enabled_results = VecDeque::from([true, true]);
        runner.tun_results = VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_per_user_tun",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_per_user_tun",
                "configure_gpu",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.prompt_admin_snapshots.len(), 2);
        assert!(runner.prompt_admin_snapshots[1].user_tun.is_empty());
        assert!(runner.prompt_admin_snapshots[1].user_dns.is_empty());
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_users,
            vec!["mcb".to_string()]
        );
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_admin_users,
            vec!["mcb".to_string()]
        );
        Ok(())
    }

    #[test]
    fn server_override_back_without_tun_returns_to_admin_step_and_clears_override_state()
    -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-server-back")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prepare_host_results =
            VecDeque::from([Ok(("demo".to_string(), HostProfileKind::Server))]);
        runner.prompt_users_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.prompt_admin_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.per_user_tun_enabled_results = VecDeque::from([false, false]);
        runner.server_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_server_overrides",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_server_overrides",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.prompt_admin_snapshots.len(), 2);
        assert!(!runner.prompt_admin_snapshots[1].server_overrides_enabled);
        assert!(
            runner.prompt_admin_snapshots[1]
                .server_enable_network_cli
                .is_empty()
        );
        assert!(
            runner.prompt_admin_snapshots[1]
                .server_enable_docker
                .is_empty()
        );
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_users,
            vec!["mcb".to_string()]
        );
        assert_eq!(
            runner.prompt_admin_snapshots[1].target_admin_users,
            vec!["mcb".to_string()]
        );
        Ok(())
    }

    #[test]
    fn summary_back_revisits_server_override_and_can_disable_previous_override() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-summary-server-loop")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prepare_host_results =
            VecDeque::from([Ok(("demo".to_string(), HostProfileKind::Server))]);
        runner.server_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);
        runner.server_enabled_results = VecDeque::from([true, false]);
        runner.summary_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_server_overrides",
                "confirm_summary",
                "configure_server_overrides",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.server_snapshots.len(), 2);
        assert!(!runner.server_snapshots[0].server_overrides_enabled);
        assert!(runner.server_snapshots[1].server_overrides_enabled);
        assert_eq!(runner.server_snapshots[1].server_enable_network_cli, "true");
        assert_eq!(runner.server_snapshots[1].server_enable_docker, "true");
        assert_eq!(runner.summary_snapshots.len(), 2);
        assert!(runner.summary_snapshots[0].server_overrides_enabled);
        assert!(
            runner.summary_snapshots[1]
                .server_enable_network_cli
                .is_empty()
        );
        assert!(runner.summary_snapshots[1].server_enable_docker.is_empty());
        assert!(!app.server_overrides_enabled);
        assert!(app.server_enable_network_cli.is_empty());
        assert!(app.server_enable_docker.is_empty());
        Ok(())
    }

    #[test]
    fn summary_back_revisits_last_runtime_step() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-summary-back")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.summary_results =
            VecDeque::from([Ok(WizardAction::Back), Ok(WizardAction::Continue)]);
        runner.gpu_results =
            VecDeque::from([Ok(WizardAction::Continue), Ok(WizardAction::Continue)]);

        wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)?;

        assert_eq!(
            runner.calls,
            vec![
                "prepare_host",
                "prompt_users",
                "prompt_admin_users",
                "per_user_tun_enabled",
                "configure_gpu",
                "confirm_summary",
                "configure_gpu",
                "confirm_summary",
            ]
        );
        assert_eq!(runner.summary_snapshots.len(), 2);
        assert_eq!(
            runner.summary_snapshots[0].host_profile_kind,
            HostProfileKind::Desktop
        );
        Ok(())
    }

    #[test]
    fn wizard_flow_stops_after_user_prompt_error() -> Result<()> {
        let repo_dir = create_temp_dir("mcbctl-wizard-user-error")?;
        let mut app = test_app(repo_dir.clone());
        let mut runner = TestWizardRunner::new();
        runner.prompt_users_results = VecDeque::from([Err(anyhow::anyhow!("prompt users failed"))]);

        let err = wizard_flow_with_runner(&mut app, &repo_dir, &mut runner)
            .expect_err("wizard should stop when prompt_users fails");

        assert!(err.to_string().contains("prompt users failed"));
        assert_eq!(runner.calls, vec!["prepare_host", "prompt_users"]);
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
            repo_urls: Vec::new(),
            branch: "rust脚本分支".to_string(),
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
