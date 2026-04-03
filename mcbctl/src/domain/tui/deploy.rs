#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeployTask {
    DirectDeploy,
    AdjustStructure,
    Maintenance,
}

impl DeployTask {
    pub const ALL: [DeployTask; 3] = [
        DeployTask::DirectDeploy,
        DeployTask::AdjustStructure,
        DeployTask::Maintenance,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DeployTask::DirectDeploy => "直接部署这台机器",
            DeployTask::AdjustStructure => "调整主机结构与用户",
            DeployTask::Maintenance => "维护与诊断",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeploySource {
    CurrentRepo,
    EtcNixos,
    RemotePinned,
    RemoteHead,
}

impl DeploySource {
    pub const ALL: [DeploySource; 4] = [
        DeploySource::CurrentRepo,
        DeploySource::EtcNixos,
        DeploySource::RemotePinned,
        DeploySource::RemoteHead,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DeploySource::CurrentRepo => "当前仓库",
            DeploySource::EtcNixos => "/etc/nixos",
            DeploySource::RemotePinned => "远端固定版本",
            DeploySource::RemoteHead => "远端最新版本",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeployAction {
    Switch,
    Test,
    Boot,
    Build,
}

impl DeployAction {
    pub const ALL: [DeployAction; 4] = [
        DeployAction::Switch,
        DeployAction::Test,
        DeployAction::Boot,
        DeployAction::Build,
    ];

    pub fn label(self) -> &'static str {
        match self {
            DeployAction::Switch => "switch",
            DeployAction::Test => "test",
            DeployAction::Boot => "boot",
            DeployAction::Build => "build",
        }
    }

    pub fn rebuild_mode(self) -> &'static str {
        match self {
            DeployAction::Switch => "switch",
            DeployAction::Test => "test",
            DeployAction::Boot => "boot",
            DeployAction::Build => "build",
        }
    }

    pub fn from_rebuild_mode(mode: &str) -> Self {
        match mode {
            "test" => DeployAction::Test,
            "boot" => DeployAction::Boot,
            "build" => DeployAction::Build,
            _ => DeployAction::Switch,
        }
    }
}
