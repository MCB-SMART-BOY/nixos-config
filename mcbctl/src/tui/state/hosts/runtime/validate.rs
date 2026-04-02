use super::*;

impl AppState {
    pub(super) fn current_host_runtime_validation_errors(&self) -> Vec<String> {
        let Some(settings) = self.current_host_settings() else {
            return vec!["当前主机没有可用设置。".to_string()];
        };

        let mut errors = Vec::new();
        if settings.proxy_mode == "http" && settings.proxy_url.trim().is_empty() {
            errors.push("proxyMode = http 时，代理 URL 不能为空。".to_string());
        }
        if settings.proxy_mode == "tun"
            && !settings.per_user_tun_enable
            && settings.tun_interface.trim().is_empty()
        {
            errors.push(
                "proxyMode = tun 且未开启 per-user TUN 时，主 TUN 接口不能为空。".to_string(),
            );
        }
        if settings.per_user_tun_enable && settings.proxy_mode != "tun" {
            errors.push("启用 per-user TUN 时，proxyMode 必须为 tun。".to_string());
        }
        if settings.per_user_tun_enable {
            for user in &settings.users {
                if !settings.per_user_tun_interfaces.contains_key(user) {
                    errors.push(format!("per-user TUN 接口映射缺少用户：{user}"));
                }
            }
        }
        if settings.gpu_mode == "hybrid" {
            let has_igpu = if settings.gpu_igpu_vendor == "amd" {
                settings
                    .gpu_amd_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            } else {
                settings
                    .gpu_intel_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            };
            let has_nvidia = settings
                .gpu_nvidia_bus
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            if !has_igpu || !has_nvidia {
                errors.push("GPU hybrid 模式要求设置 nvidiaBusId 和匹配的 iGPU Bus ID。".to_string());
            }
        }
        if settings.gpu_specialisations_enable
            && settings
                .gpu_specialisation_modes
                .iter()
                .any(|mode| mode == "hybrid")
        {
            let has_igpu = if settings.gpu_igpu_vendor == "amd" {
                settings
                    .gpu_amd_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            } else {
                settings
                    .gpu_intel_bus
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            };
            let has_nvidia = settings
                .gpu_nvidia_bus
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty());
            if !has_igpu || !has_nvidia {
                errors.push("GPU 特化包含 hybrid 时，需要配置完整的 PRIME Bus ID。".to_string());
            }
        }

        errors
    }
}
