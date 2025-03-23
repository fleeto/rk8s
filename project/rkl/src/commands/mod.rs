use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use libcgroups::common::AnyCgroupManager;
use libcontainer::container::Container;

pub mod create;
pub mod start;
pub mod delete;
pub mod state;
pub mod kill;


fn construct_container_root<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<PathBuf> {
    // resolves relative paths, symbolic links etc. and get complete path
    let root_path = fs::canonicalize(&root_path).with_context(|| {
        format!(
            "failed to canonicalize {} for container {}",
            root_path.as_ref().display(),
            container_id
        )
    })?;
    // the state of the container is stored in a directory named after the container id
    Ok(root_path.join(container_id))
}

pub fn load_container<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<Container> {
    let container_root = construct_container_root(root_path, container_id)?;
    if !container_root.exists() {
        bail!("container {} does not exist.", container_id)
    }

    Container::load(container_root)
        .with_context(|| format!("could not load state for container {container_id}"))
}

fn container_exists<P: AsRef<Path>>(root_path: P, container_id: &str) -> Result<bool> {
    let container_root = construct_container_root(root_path, container_id)?;
    Ok(container_root.exists())
}

fn create_cgroup_manager<P: AsRef<Path>>(
    root_path: P,
    container_id: &str,
) -> Result<AnyCgroupManager> {
    let container = load_container(root_path, container_id)?;
    Ok(libcgroups::common::create_cgroup_manager(
        libcgroups::common::CgroupConfig {
            cgroup_path: container.spec()?.cgroup_path,
            systemd_cgroup: container.systemd(),
            container_name: container.id().to_string(),
        },
    )?)
}
