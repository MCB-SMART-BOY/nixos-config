use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub group: Option<String>,
    pub expr: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub workflow_tags: Vec<String>,
    #[serde(default)]
    pub lifecycle: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub desktop_entry_flag: Option<String>,
}

impl CatalogEntry {
    pub fn group_key(&self) -> &str {
        self.group.as_deref().unwrap_or(&self.category)
    }

    pub fn source_label(&self) -> &str {
        self.source.as_deref().unwrap_or("nixpkgs")
    }

    pub fn matches(&self, category: Option<&str>, query: &str) -> bool {
        if let Some(category) = category
            && self.category != category
        {
            return false;
        }

        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }

        let haystack = format!(
            "{} {} {} {} {} {} {} {} {} {} {}",
            self.id,
            self.name,
            self.category,
            self.group_key(),
            self.expr,
            self.description.as_deref().unwrap_or(""),
            self.source_label(),
            self.keywords.join(" "),
            self.workflow_tags.join(" "),
            self.lifecycle.as_deref().unwrap_or(""),
            self.platforms.join(" ")
        )
        .to_lowercase();
        haystack.contains(&query)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct GroupMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub order: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HomeOptionMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_home_option_area")]
    pub area: String,
    #[serde(default)]
    pub order: u32,
}

fn default_home_option_area() -> String {
    "desktop".to_string()
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkflowMeta {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub order: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_includes_workflow_tags_and_lifecycle() {
        let entry = CatalogEntry {
            id: "ollama".to_string(),
            name: "Ollama".to_string(),
            category: "ai".to_string(),
            group: Some("ai-tools".to_string()),
            expr: "pkgs.ollama".to_string(),
            description: Some("本地模型".to_string()),
            keywords: vec!["llm".to_string()],
            workflow_tags: vec!["ai".to_string()],
            lifecycle: Some("stable".to_string()),
            source: Some("nixpkgs".to_string()),
            platforms: vec!["x86_64-linux".to_string()],
            desktop_entry_flag: None,
        };

        assert!(entry.matches(None, "ai"));
        assert!(entry.matches(None, "stable"));
    }
}
