use super::*;

impl App {
    pub(crate) fn add_target_user(&mut self, user: &str) {
        if !self.target_users.iter().any(|u| u == user) {
            self.target_users.push(user.to_string());
        }
    }

    pub(crate) fn remove_target_user(&mut self, user: &str) {
        self.target_users.retain(|u| u != user);
    }

    pub(crate) fn toggle_target_user(&mut self, user: &str) {
        if self.target_users.iter().any(|u| u == user) {
            self.remove_target_user(user);
        } else {
            self.add_target_user(user);
        }
    }

    pub(crate) fn add_admin_user(&mut self, user: &str) {
        if !self.target_admin_users.iter().any(|u| u == user) {
            self.target_admin_users.push(user.to_string());
        }
    }

    pub(crate) fn remove_admin_user(&mut self, user: &str) {
        self.target_admin_users.retain(|u| u != user);
    }

    pub(crate) fn toggle_admin_user(&mut self, user: &str) {
        if self.target_admin_users.iter().any(|u| u == user) {
            self.remove_admin_user(user);
        } else {
            self.add_admin_user(user);
        }
    }

    pub(crate) fn dedupe_users(&mut self) {
        let mut set = BTreeSet::new();
        let mut out = Vec::new();
        for u in &self.target_users {
            if set.insert(u.clone()) {
                out.push(u.clone());
            }
        }
        self.target_users = out;
    }

    pub(crate) fn dedupe_admin_users(&mut self) {
        let mut set = BTreeSet::new();
        let mut out = Vec::new();
        for u in &self.target_admin_users {
            if set.insert(u.clone()) {
                out.push(u.clone());
            }
        }
        self.target_admin_users = out;
    }
}
