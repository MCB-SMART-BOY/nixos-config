use super::*;

#[path = "runtime/gpu.rs"]
mod gpu;
#[path = "runtime/server.rs"]
mod server;
#[path = "runtime/tun.rs"]
mod tun;

impl App {
    pub(crate) fn reset_admin_users(&mut self) {
        self.target_admin_users.clear();
    }
}
