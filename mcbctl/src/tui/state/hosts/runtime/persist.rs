use super::*;

impl AppState {
    pub fn save_current_host_runtime(&mut self) -> Result<()> {
        let errors = self.current_host_runtime_validation_errors();
        if !errors.is_empty() {
            self.status = format!("当前主机的运行时分片未通过校验：{}", errors.join("；"));
            return Ok(());
        }

        let host = self.target_host.clone();
        let Some(settings) = self.current_host_settings().cloned() else {
            self.status = "没有可保存的主机运行时配置。".to_string();
            return Ok(());
        };

        let host_dir = self.context.repo_root.join("hosts").join(&host);
        let managed_dir = host_dir.join("managed");
        ensure_managed_host_layout(&managed_dir)?;
        let paths = write_host_runtime_fragments(&managed_dir, &settings)?;
        self.host_dirty_runtime_hosts.remove(&host);
        self.status = format!(
            "已写入 {}",
            paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join("、")
        );
        Ok(())
    }
}
