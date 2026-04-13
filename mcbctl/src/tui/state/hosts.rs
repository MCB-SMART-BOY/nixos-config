use super::*;
use anyhow::Result;

use self::runtime::validate_host_runtime_settings;
use self::users::validate_host_user_settings;

mod runtime;
mod users;

impl AppState {
    pub(super) fn host_settings(&self, host: &str) -> Option<&HostManagedSettings> {
        self.host_settings_by_name.get(host)
    }

    pub(super) fn host_configuration_validation_errors(&self, host: &str) -> Vec<String> {
        let Some(settings) = self.host_settings(host) else {
            return vec![format!("主机 {host} 没有可用配置。")];
        };

        let mut errors = validate_host_user_settings(settings);
        errors.extend(validate_host_runtime_settings(settings));
        errors
    }

    pub(super) fn current_host_settings(&self) -> Option<&HostManagedSettings> {
        self.host_settings(&self.target_host)
    }

    fn current_host_settings_mut(&mut self) -> Option<&mut HostManagedSettings> {
        self.host_settings_by_name.get_mut(&self.target_host)
    }

    pub(crate) fn ensure_host_configuration_is_valid(&self, host: &str) -> Result<()> {
        let errors = self.host_configuration_validation_errors(host);
        if errors.is_empty() {
            return Ok(());
        }

        anyhow::bail!("主机 {host} 的 TUI 配置未通过校验：{}", errors.join("；"))
    }

    pub fn current_host_users_path(&self) -> Option<PathBuf> {
        let host = self
            .context
            .hosts
            .iter()
            .find(|name| *name == &self.target_host)?;
        Some(managed_host_users_path(&self.context.repo_root, host))
    }

    pub fn current_host_runtime_paths(&self) -> Vec<PathBuf> {
        let Some(host) = self
            .context
            .hosts
            .iter()
            .find(|name| *name == &self.target_host)
        else {
            return Vec::new();
        };

        vec![
            managed_host_network_path(&self.context.repo_root, host),
            managed_host_gpu_path(&self.context.repo_root, host),
            managed_host_virtualization_path(&self.context.repo_root, host),
        ]
    }

    pub fn switch_target_host(&mut self, delta: i8) {
        cycle_string(&mut self.target_host, &self.context.hosts, delta);
    }
}
