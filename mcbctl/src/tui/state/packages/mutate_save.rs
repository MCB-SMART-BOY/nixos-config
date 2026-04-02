use super::*;

impl AppState {
    pub fn save_current_user_packages(&mut self) -> Result<()> {
        let Some(user) = self.current_package_user().map(ToOwned::to_owned) else {
            self.status = "没有可保存的用户。".to_string();
            return Ok(());
        };

        let selected = self
            .package_user_selections
            .get(&user)
            .cloned()
            .unwrap_or_default();
        let managed_dir = self
            .context
            .repo_root
            .join("home/users")
            .join(&user)
            .join("managed");
        ensure_managed_packages_layout(&managed_dir)?;
        write_grouped_managed_packages(&managed_dir, &self.context.catalog_entries, &selected)?;
        self.package_dirty_users.remove(&user);
        self.status = format!("已写入 {}", managed_dir.join("packages").display());
        Ok(())
    }
}
