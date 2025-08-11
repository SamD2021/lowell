use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub name: String,
    pub root: String,         // ostree | composefs | plain
    pub modules: Vec<String>, // ["virtio_blk","virtio_net","xfs","ext4"]
    pub cmdline: Option<String>, // "console=ttyS0"
                              // TODO: add kernel/userspace artifact refs later
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_profile_from_toml() {
        let s = r#"
            name = "kvm-ostree"
            root = "ostree"
            modules = ["virtio_blk","virtio_net","xfs","ext4"]
            cmdline = "console=ttyS0,115200n8"
        "#;
        let p: Profile = toml::from_str(s).expect("valid profile");
        assert_eq!(p.name, "kvm-ostree");
        assert_eq!(p.root, "ostree");
        assert_eq!(p.modules, vec!["virtio_blk", "virtio_net", "xfs", "ext4"]);
        assert_eq!(p.cmdline.as_deref(), Some("console=ttyS0,115200n8"));
    }

    #[test]
    fn deserialize_profile_without_cmdline() {
        let s = r#"
            name = "kvm-composefs"
            root = "composefs"
            modules = ["virtio_blk","virtio_net"]
        "#;
        let p: Profile = toml::from_str(s).expect("valid profile");
        assert_eq!(p.name, "kvm-composefs");
        assert_eq!(p.root, "composefs");
        assert!(p.cmdline.is_none());
    }
}
