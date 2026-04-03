use super::*;

impl App {
    pub(crate) fn validate_users(&self) -> Result<()> {
        for user in &self.target_users {
            if !is_valid_username(user) {
                bail!("用户名不合法：{user}");
            }
        }
        Ok(())
    }

    pub(crate) fn validate_admin_users(&mut self) -> Result<()> {
        if self.target_admin_users.is_empty() && !self.target_users.is_empty() {
            self.target_admin_users = vec![self.target_users[0].clone()];
        }
        for user in &self.target_admin_users {
            if !is_valid_username(user) {
                bail!("管理员用户名不合法：{user}");
            }
            if !self.target_users.iter().any(|u| u == user) {
                bail!("管理员用户必须包含在用户列表中：{user}");
            }
        }
        Ok(())
    }
}
