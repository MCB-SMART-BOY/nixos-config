#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageTextMode {
    Search,
    CreateGroup,
    RenameGroup,
    ConfirmWorkflowAdd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackageDataMode {
    Local,
    Search,
}

impl PackageDataMode {
    pub fn label(self) -> &'static str {
        match self {
            PackageDataMode::Local => "本地覆盖/已声明",
            PackageDataMode::Search => "nixpkgs 搜索",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsersTextMode {
    ManagedUsers,
    AdminUsers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeployTextMode {
    ApplyRemotePinnedRef,
    AdvancedWizardRemotePinnedRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostsTextMode {
    CustomSubstituters,
    CustomTrustedPublicKeys,
    ProxyUrl,
    TunInterface,
    TunInterfaces,
    ProxyDnsAddr,
    ProxyDnsPort,
    PerUserTunInterfaces,
    PerUserTunDnsPorts,
    PerUserTunTableBase,
    PerUserTunPriorityBase,
    IntelBusId,
    AmdBusId,
    NvidiaBusId,
    SpecialisationModes,
}
