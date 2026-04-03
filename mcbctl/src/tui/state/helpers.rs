use super::*;

pub(super) fn cycle_enum<T: Copy + Eq>(current: &mut T, all: &[T], delta: i8) {
    let Some(index) = all.iter().position(|item| item == current) else {
        return;
    };
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    *current = all[next];
}

pub(super) fn bool_label(value: bool) -> &'static str {
    if value { "开启" } else { "关闭" }
}

pub(super) fn default_target_host(context: &AppContext) -> String {
    if context
        .hosts
        .iter()
        .any(|host| host == &context.current_host)
    {
        return context.current_host.clone();
    }
    if context.hosts.iter().any(|host| host == "nixos") {
        return "nixos".to_string();
    }
    context
        .hosts
        .first()
        .cloned()
        .unwrap_or_else(|| context.current_host.clone())
}

pub(super) fn default_package_user_index(
    context: &AppContext,
    target_host: &str,
    host_settings_by_name: &BTreeMap<String, HostManagedSettings>,
) -> usize {
    if let Some(index) = context
        .users
        .iter()
        .position(|user| user == &context.current_user)
    {
        return index;
    }

    if let Some(primary_user) = host_settings_by_name
        .get(target_host)
        .map(|settings| settings.primary_user.trim())
        .filter(|user| !user.is_empty())
        && let Some(index) = context.users.iter().position(|user| user == primary_user)
    {
        return index;
    }
    0
}

pub(super) fn format_string_list(items: &[String]) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items.join(", ")
    }
}

pub(super) fn serialize_string_list(items: &[String]) -> String {
    items.join(", ")
}

pub(super) fn format_string_map(items: &BTreeMap<String, String>) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub(super) fn serialize_string_map(items: &BTreeMap<String, String>) -> String {
    items
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn format_u16_map(items: &BTreeMap<String, u16>) -> String {
    if items.is_empty() {
        "无".to_string()
    } else {
        items
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub(super) fn serialize_u16_map(items: &BTreeMap<String, u16>) -> String {
    items
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn parse_string_list(raw: &str) -> Vec<String> {
    dedup_string_list(
        raw.split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToOwned::to_owned)
            .collect(),
    )
}

pub(super) fn dedup_string_list(items: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in items {
        if !output.contains(&item) {
            output.push(item);
        }
    }
    output
}

pub(super) fn has_duplicates(items: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    for item in items {
        if !seen.insert(item) {
            return true;
        }
    }
    false
}

pub(super) fn parse_string_map(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut output = BTreeMap::new();
    if raw.trim().is_empty() {
        return Ok(output);
    }

    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((key, value)) = piece.split_once('=') else {
            anyhow::bail!("映射项必须是 user=value 形式：{piece}");
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            anyhow::bail!("映射项不能为空：{piece}");
        }
        output.insert(key.to_string(), value.to_string());
    }

    Ok(output)
}

pub(super) fn parse_u16_map(raw: &str) -> Result<BTreeMap<String, u16>> {
    let mut output = BTreeMap::new();
    if raw.trim().is_empty() {
        return Ok(output);
    }

    for part in raw.split(',') {
        let piece = part.trim();
        if piece.is_empty() {
            continue;
        }
        let Some((key, value)) = piece.split_once('=') else {
            anyhow::bail!("端口映射必须是 user=1053 形式：{piece}");
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            anyhow::bail!("端口映射项不能为空：{piece}");
        }
        let port = value
            .parse::<u16>()
            .with_context(|| format!("无效端口：{value}"))?;
        output.insert(key.to_string(), port);
    }

    Ok(output)
}

pub(super) fn parse_gpu_modes(raw: &str) -> Result<Vec<String>> {
    let modes = parse_string_list(raw);
    for mode in &modes {
        if !matches!(mode.as_str(), "igpu" | "hybrid" | "dgpu") {
            anyhow::bail!("无效 GPU 特化模式：{mode}");
        }
    }
    Ok(modes)
}

pub(super) fn empty_to_none(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value.trim().to_string())
    }
}

pub(super) fn nonempty_label(value: &str) -> String {
    if value.trim().is_empty() {
        "无".to_string()
    } else {
        value.to_string()
    }
}

pub(super) fn nonempty_opt_label(value: Option<&str>) -> String {
    value
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "无".to_string())
}

pub(super) fn normalize_package_group_name(input: &str) -> String {
    let mut output = String::new();
    let mut last_was_dash = false;

    for ch in input.chars().flat_map(char::to_lowercase) {
        let mapped = match ch {
            'a'..='z' | '0'..='9' => Some(ch),
            '-' | '_' | ' ' | '/' | '.' => Some('-'),
            _ => None,
        };

        let Some(ch) = mapped else {
            continue;
        };

        if ch == '-' {
            if output.is_empty() || last_was_dash {
                continue;
            }
            last_was_dash = true;
            output.push(ch);
        } else {
            last_was_dash = false;
            output.push(ch);
        }
    }

    while output.ends_with('-') {
        output.pop();
    }

    output
}

pub(super) fn display_path(path: Option<PathBuf>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "无".to_string())
}

pub(super) fn is_local_overlay_entry(entry: &CatalogEntry) -> bool {
    let source = entry.source_label();
    source.starts_with("local/") || source.starts_with("overlay/") || source.starts_with("managed/")
}

pub(super) fn refresh_local_catalog_indexes(
    context: &mut AppContext,
    local_entry_ids: &BTreeSet<String>,
) {
    let mut categories = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for entry in &context.catalog_entries {
        if !local_entry_ids.contains(&entry.id) {
            continue;
        }
        categories.insert(entry.category.clone());
        sources.insert(entry.source_label().to_string());
    }

    context.catalog_categories = categories.into_iter().collect();
    context.catalog_sources = sources.into_iter().collect();
}

pub(super) fn cycle_string_value(current: &str, all: &[String], delta: i8) -> Option<String> {
    if all.is_empty() {
        return None;
    }
    let index = all.iter().position(|item| item == current).unwrap_or(0);
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    Some(all[next].clone())
}

pub(super) fn cycle_string(current: &mut String, all: &[String], delta: i8) {
    if all.is_empty() {
        return;
    }
    let index = all.iter().position(|item| item == current).unwrap_or(0);
    let len = all.len() as isize;
    let next = (index as isize + delta as isize).rem_euclid(len) as usize;
    *current = all[next].clone();
}
