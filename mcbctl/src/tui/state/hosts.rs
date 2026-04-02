use super::*;

mod runtime;
mod users;

impl AppState {
    pub(super) fn current_host_settings(&self) -> Option<&HostManagedSettings> {
        self.host_settings_by_name.get(&self.target_host)
    }

    fn current_host_settings_mut(&mut self) -> Option<&mut HostManagedSettings> {
        self.host_settings_by_name.get_mut(&self.target_host)
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
