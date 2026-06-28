use crate::overlay::errors::OverlayError;
use crate::overlay::model::{Group, Layout, OverlayConfig};
use std::collections::HashMap;
use tokio::sync::Mutex;

const MAX_GROUPS: usize = 20;

/// In-memory store of overlay groups and per-countdown styling, guarded by
/// async mutexes (mirrors [`CountdownService`](crate::countdown::service::CountdownService)).
#[derive(Debug)]
pub struct OverlayService {
    groups: Mutex<HashMap<u64, Group>>,
    configs: Mutex<HashMap<u64, OverlayConfig>>,
    next_id: Mutex<u64>,
}

impl Default for OverlayService {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayService {
    pub fn new() -> Self {
        Self {
            groups: Mutex::new(HashMap::new()),
            configs: Mutex::new(HashMap::new()),
            next_id: Mutex::new(0),
        }
    }

    pub async fn create_group(&self, name: String) -> Result<u64, OverlayError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(OverlayError::EmptyName);
        }
        let mut groups = self.groups.lock().await;
        if groups.len() >= MAX_GROUPS {
            return Err(OverlayError::MaxGroupsReached);
        }
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;
        groups.insert(
            id,
            Group {
                id,
                name,
                members: Vec::new(),
                layout: Layout::default(),
                hide_idle: false,
            },
        );
        Ok(id)
    }

    pub async fn list_groups(&self) -> Vec<Group> {
        let mut groups: Vec<Group> = self.groups.lock().await.values().cloned().collect();
        groups.sort_by_key(|g| g.id);
        groups
    }

    pub async fn get_group(&self, id: u64) -> Option<Group> {
        self.groups.lock().await.get(&id).cloned()
    }

    pub async fn update_group(
        &self,
        id: u64,
        name: String,
        members: Vec<u64>,
        layout: Layout,
        hide_idle: bool,
    ) -> Result<(), OverlayError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(OverlayError::EmptyName);
        }
        let mut groups = self.groups.lock().await;
        let group = groups.get_mut(&id).ok_or(OverlayError::GroupNotFound)?;
        group.name = name;
        group.members = members;
        group.layout = layout;
        group.hide_idle = hide_idle;
        Ok(())
    }

    pub async fn delete_group(&self, id: u64) -> Result<(), OverlayError> {
        if self.groups.lock().await.remove(&id).is_some() {
            Ok(())
        } else {
            Err(OverlayError::GroupNotFound)
        }
    }

    pub async fn set_config(&self, id: u64, config: OverlayConfig) {
        self.configs.lock().await.insert(id, config);
    }

    pub async fn get_config(&self, id: u64) -> OverlayConfig {
        self.configs
            .lock()
            .await
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }
}
