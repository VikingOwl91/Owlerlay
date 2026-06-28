use crate::overlay::model::{Group, Layout};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDto {
    pub id: u64,
    pub name: String,
    pub members: Vec<u64>,
    pub layout: Layout,
    pub hide_idle: bool,
}

impl From<Group> for GroupDto {
    fn from(g: Group) -> Self {
        Self {
            id: g.id,
            name: g.name,
            members: g.members,
            layout: g.layout,
            hide_idle: g.hide_idle,
        }
    }
}
