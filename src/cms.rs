//! Wall for the console. Plane store is the database; this crate only shapes it.

use connect_control_plane::store::{Kind, Store};
use serde::Serialize;

#[derive(Clone)]
pub struct Who {
    pub email: String,
    pub org: String,
    pub name: String,
    pub role: String,
    pub personal: bool,
    pub scope: String,
}

impl Who {
    pub fn admin(&self) -> bool {
        self.role == "owner" || self.role == "admin"
    }

    pub fn owner(&self) -> bool {
        self.role == "owner"
    }

    pub fn may_box(&self, owner: &str) -> bool {
        self.admin() || owner == self.name
    }
}

#[derive(Serialize, Clone, Default)]
pub struct Record {
    pub id: String,
    pub cells: Vec<String>,
}

#[derive(Serialize, Clone, Default)]
pub struct Collection {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub hint: String,
    pub columns: Vec<String>,
    pub records: Vec<Record>,
}

#[derive(Serialize, Clone, Default)]
pub struct Wall {
    pub org: String,
    pub me: String,
    pub role: String,
    pub admin: bool,
    pub collections: Vec<Collection>,
}

pub async fn wall(store: &Store, who: &Who) -> anyhow::Result<Wall> {
    let org = who.org.as_str();
    let key = who.scope.as_str();
    let agents = store.list_agents(key).await?;
    let online = store.list_online(key).await.unwrap_or_default();
    let admin = who.admin();
    let mut collections = Vec::new();

    let mut boxes = Collection {
        id: "boxes".into(),
        title: if admin {
            "Boxes".into()
        } else {
            "Your boxes".into()
        },
        hint: "A box is a machine that dials the plane. Add one, copy the token, run the agent."
            .into(),
        columns: vec!["Name".into(), "Status".into()],
        records: Vec::new(),
    };
    for a in agents.iter().filter(|a| a.kind == Kind::Box && !a.revoked) {
        if !admin && a.owner != who.name {
            continue;
        }
        let on = online.iter().any(|o| o.agent_id == a.agent_id);
        let status = if on { "online" } else { "offline" };
        boxes.records.push(Record {
            id: a.agent_id.to_string(),
            cells: vec![a.name.clone(), status.into()],
        });
    }
    collections.push(boxes);

    if admin {
        let mut people = Collection {
            id: "people".into(),
            title: "People".into(),
            hint: "Who can open the console. Boxes they may use is a separate grant.".into(),
            columns: vec!["Name".into(), "Role".into()],
            records: Vec::new(),
        };
        for a in agents.iter().filter(|a| a.kind == Kind::Client && !a.revoked) {
            let role = if a.org_role.is_empty() {
                "member"
            } else {
                a.org_role.as_str()
            };
            people.records.push(Record {
                id: a.agent_id.to_string(),
                cells: vec![a.name.clone(), role.into()],
            });
        }
        collections.push(people);

        let mut access = Collection {
            id: "access".into(),
            title: "Access".into(),
            hint: "Who may App to which box from a laptop.".into(),
            columns: vec!["Person".into(), "Boxes".into()],
            records: Vec::new(),
        };
        for a in agents.iter().filter(|a| a.kind == Kind::Client && !a.revoked) {
            let (restricted, boxes) = store
                .list_grants(key, &a.name)
                .await
                .unwrap_or((false, Vec::new()));
            let what = if !restricted {
                "all boxes".into()
            } else if boxes.is_empty() {
                "none".into()
            } else {
                boxes.join(", ")
            };
            access.records.push(Record {
                id: format!("access-{}", a.name),
                cells: vec![a.name.clone(), what],
            });
        }
        collections.push(access);
    }

    Ok(Wall {
        org: org.into(),
        me: who.name.clone(),
        role: who.role.clone(),
        admin,
        collections,
    })
}
