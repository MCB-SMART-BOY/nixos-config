use super::*;

impl AppState {
    pub(super) fn current_host_runtime_validation_errors(&self) -> Vec<String> {
        let Some(settings) = self.current_host_settings() else {
            return vec!["当前主机没有可用设置。".to_string()];
        };

        validate_host_runtime_settings(settings)
    }
}

pub(crate) fn validate_host_runtime_settings(settings: &HostManagedSettings) -> Vec<String> {
    let mut errors = Vec::new();

    if !matches!(
        settings.cache_profile.as_str(),
        "cn" | "global" | "official-only" | "custom"
    ) {
        errors.push("cacheProfile 只能是 cn / global / official-only / custom。".to_string());
    }
    if settings.cache_profile == "custom" && settings.custom_substituters.is_empty() {
        errors.push("cacheProfile = custom 时，customSubstituters 不能为空。".to_string());
    }
    if settings.cache_profile == "custom" && settings.custom_trusted_public_keys.is_empty() {
        errors.push("cacheProfile = custom 时，customTrustedPublicKeys 不能为空。".to_string());
    }

    if !matches!(settings.proxy_mode.as_str(), "tun" | "http" | "off") {
        errors.push("proxyMode 只能是 tun / http / off。".to_string());
    }
    if settings.proxy_mode == "http" && settings.proxy_url.trim().is_empty() {
        errors.push("proxyMode = http 时，代理 URL 不能为空。".to_string());
    }
    if settings.proxy_mode == "tun"
        && !settings.per_user_tun_enable
        && settings.tun_interface.trim().is_empty()
    {
        errors.push("proxyMode = tun 且未开启 per-user TUN 时，主 TUN 接口不能为空。".to_string());
    }
    if settings.enable_proxy_dns && settings.proxy_dns_addr.trim().is_empty() {
        errors.push("启用全局代理 DNS 时，proxyDnsAddr 不能为空。".to_string());
    }
    if settings.per_user_tun_enable && settings.proxy_mode != "tun" {
        errors.push("启用 per-user TUN 时，proxyMode 必须为 tun。".to_string());
    }
    if settings.per_user_tun_enable && settings.enable_proxy_dns {
        errors.push("启用 per-user TUN 时，必须关闭全局代理 DNS。".to_string());
    }
    if settings.per_user_tun_redirect_dns && !settings.per_user_tun_enable {
        errors.push("启用 per-user DNS 重定向前，必须先启用 per-user TUN。".to_string());
    }
    if settings.per_user_tun_table_base <= 0 {
        errors.push("per-user 路由表基值必须大于 0。".to_string());
    }
    if settings.per_user_tun_priority_base <= 0 {
        errors.push("per-user 规则优先级基值必须大于 0。".to_string());
    }
    let unknown_interface_users = extra_keys(&settings.per_user_tun_interfaces, &settings.users);
    if !unknown_interface_users.is_empty() {
        errors.push(format!(
            "per-user TUN 接口映射包含不在托管用户列表中的用户：{}",
            unknown_interface_users.join(", ")
        ));
    }
    let unknown_dns_port_users = extra_keys(&settings.per_user_tun_dns_ports, &settings.users);
    if !unknown_dns_port_users.is_empty() {
        errors.push(format!(
            "per-user DNS 端口映射包含不在托管用户列表中的用户：{}",
            unknown_dns_port_users.join(", ")
        ));
    }
    if settings.per_user_tun_enable {
        for user in &settings.users {
            if !settings.per_user_tun_interfaces.contains_key(user) {
                errors.push(format!("per-user TUN 接口映射缺少用户：{user}"));
            }
        }
        if has_duplicate_values(&settings.per_user_tun_interfaces) {
            errors.push("per-user TUN 接口映射必须为每个用户使用唯一接口名。".to_string());
        }
    }
    if settings.per_user_tun_redirect_dns {
        for user in &settings.users {
            if !settings.per_user_tun_dns_ports.contains_key(user) {
                errors.push(format!("per-user DNS 端口映射缺少用户：{user}"));
            }
        }
        if has_duplicate_u16_values(&settings.per_user_tun_dns_ports) {
            errors.push("per-user DNS 端口映射必须为每个用户使用唯一端口。".to_string());
        }
    }

    if !matches!(settings.gpu_mode.as_str(), "igpu" | "hybrid" | "dgpu") {
        errors.push("GPU 模式只能是 igpu / hybrid / dgpu。".to_string());
    }
    if !matches!(settings.gpu_igpu_vendor.as_str(), "intel" | "amd") {
        errors.push("iGPU 厂商只能是 intel / amd。".to_string());
    }
    if !matches!(
        settings.gpu_prime_mode.as_str(),
        "offload" | "sync" | "reverseSync"
    ) {
        errors.push("PRIME 模式只能是 offload / sync / reverseSync。".to_string());
    }
    let invalid_specialisation_modes = invalid_gpu_modes(&settings.gpu_specialisation_modes);
    if !invalid_specialisation_modes.is_empty() {
        errors.push(format!(
            "GPU 特化模式包含无效值：{}",
            invalid_specialisation_modes.join(", ")
        ));
    }
    if settings.gpu_specialisations_enable && settings.gpu_specialisation_modes.is_empty() {
        errors.push("启用 GPU 特化时，至少需要一个特化模式。".to_string());
    }
    if settings.gpu_mode == "hybrid" && !has_complete_hybrid_bus_ids(settings) {
        errors.push("GPU hybrid 模式要求设置 nvidiaBusId 和匹配的 iGPU Bus ID。".to_string());
    }
    if settings.gpu_specialisations_enable
        && settings
            .gpu_specialisation_modes
            .iter()
            .any(|mode| mode == "hybrid")
        && !has_complete_hybrid_bus_ids(settings)
    {
        errors.push("GPU 特化包含 hybrid 时，需要配置完整的 PRIME Bus ID。".to_string());
    }

    errors
}

fn has_complete_hybrid_bus_ids(settings: &HostManagedSettings) -> bool {
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
    has_igpu && has_nvidia
}

fn has_duplicate_values(values: &BTreeMap<String, String>) -> bool {
    let mut seen = BTreeSet::new();
    for value in values.values() {
        if !seen.insert(value.trim()) {
            return true;
        }
    }
    false
}

fn has_duplicate_u16_values(values: &BTreeMap<String, u16>) -> bool {
    let mut seen = BTreeSet::new();
    for value in values.values() {
        if !seen.insert(*value) {
            return true;
        }
    }
    false
}

fn extra_keys<T>(values: &BTreeMap<String, T>, allowed_users: &[String]) -> Vec<String> {
    values
        .keys()
        .filter(|user| !allowed_users.contains(user))
        .cloned()
        .collect()
}

fn invalid_gpu_modes(modes: &[String]) -> Vec<String> {
    modes
        .iter()
        .filter(|mode| !matches!(mode.as_str(), "igpu" | "hybrid" | "dgpu"))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_custom_cache_without_required_lists() {
        let settings = HostManagedSettings {
            cache_profile: "custom".to_string(),
            ..HostManagedSettings::default()
        };

        let errors = validate_host_runtime_settings(&settings);
        assert!(
            errors
                .iter()
                .any(|err| err.contains("customSubstituters 不能为空"))
        );
        assert!(
            errors
                .iter()
                .any(|err| err.contains("customTrustedPublicKeys 不能为空"))
        );
    }

    #[test]
    fn rejects_invalid_route_bases_and_stale_user_maps() {
        let mut settings = HostManagedSettings {
            users: vec!["alice".to_string()],
            per_user_tun_enable: true,
            per_user_tun_table_base: 0,
            per_user_tun_priority_base: -1,
            ..HostManagedSettings::default()
        };
        settings
            .per_user_tun_interfaces
            .insert("alice".to_string(), "tun-a".to_string());
        settings
            .per_user_tun_interfaces
            .insert("bob".to_string(), "tun-b".to_string());
        settings
            .per_user_tun_dns_ports
            .insert("bob".to_string(), 1053);

        let errors = validate_host_runtime_settings(&settings);
        assert!(
            errors
                .iter()
                .any(|err| err.contains("路由表基值必须大于 0"))
        );
        assert!(
            errors
                .iter()
                .any(|err| err.contains("规则优先级基值必须大于 0"))
        );
        assert!(
            errors
                .iter()
                .any(|err| err.contains("接口映射包含不在托管用户列表中"))
        );
        assert!(
            errors
                .iter()
                .any(|err| err.contains("DNS 端口映射包含不在托管用户列表中"))
        );
    }

    #[test]
    fn rejects_per_user_tun_with_global_dns_and_duplicate_maps() {
        let mut settings = HostManagedSettings {
            users: vec!["alice".to_string(), "bob".to_string()],
            proxy_mode: "tun".to_string(),
            enable_proxy_dns: true,
            per_user_tun_enable: true,
            per_user_tun_redirect_dns: true,
            ..HostManagedSettings::default()
        };
        settings
            .per_user_tun_interfaces
            .insert("alice".to_string(), "tun0".to_string());
        settings
            .per_user_tun_interfaces
            .insert("bob".to_string(), "tun0".to_string());
        settings
            .per_user_tun_dns_ports
            .insert("alice".to_string(), 1053);
        settings
            .per_user_tun_dns_ports
            .insert("bob".to_string(), 1053);

        let errors = validate_host_runtime_settings(&settings);
        assert!(
            errors
                .iter()
                .any(|err| err.contains("必须关闭全局代理 DNS"))
        );
        assert!(errors.iter().any(|err| err.contains("唯一接口名")));
        assert!(errors.iter().any(|err| err.contains("唯一端口")));
    }

    #[test]
    fn rejects_hybrid_without_required_bus_ids() {
        let settings = HostManagedSettings {
            gpu_mode: "hybrid".to_string(),
            gpu_specialisations_enable: true,
            gpu_specialisation_modes: vec!["hybrid".to_string()],
            ..HostManagedSettings::default()
        };

        let errors = validate_host_runtime_settings(&settings);
        assert!(errors.iter().any(|err| err.contains("GPU hybrid 模式")));
        assert!(errors.iter().any(|err| err.contains("GPU 特化包含 hybrid")));
    }

    #[test]
    fn rejects_invalid_gpu_enum_values() {
        let settings = HostManagedSettings {
            gpu_mode: "sidecar".to_string(),
            gpu_igpu_vendor: "nvidia".to_string(),
            gpu_prime_mode: "mux".to_string(),
            gpu_specialisations_enable: true,
            gpu_specialisation_modes: vec!["igpu".to_string(), "mystery".to_string()],
            ..HostManagedSettings::default()
        };

        let errors = validate_host_runtime_settings(&settings);
        assert!(errors.iter().any(|err| err.contains("GPU 模式只能是")));
        assert!(errors.iter().any(|err| err.contains("iGPU 厂商只能是")));
        assert!(errors.iter().any(|err| err.contains("PRIME 模式只能是")));
        assert!(
            errors
                .iter()
                .any(|err| err.contains("GPU 特化模式包含无效值"))
        );
    }

    #[test]
    fn rejects_empty_gpu_specialisation_list_when_enabled() {
        let settings = HostManagedSettings {
            gpu_specialisations_enable: true,
            gpu_specialisation_modes: Vec::new(),
            ..HostManagedSettings::default()
        };

        let errors = validate_host_runtime_settings(&settings);
        assert!(
            errors
                .iter()
                .any(|err| err.contains("至少需要一个特化模式"))
        );
    }
}
