use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Space {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub machines: Option<u32>,
    #[serde(default)]
    pub online: Option<u32>,
    #[serde(default)]
    pub people: Option<u32>,
}

impl Space {
    pub fn label(&self) -> String {
        if self.name == "personal" {
            "Personal".into()
        } else {
            self.name.clone()
        }
    }

    pub fn href(&self) -> String {
        space_href(&self.name)
    }
}

pub fn space_href(org: &str) -> String {
    if org == "personal" {
        "/personal".into()
    } else {
        format!("/org/{org}")
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Me {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub org: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub display: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub personal: bool,
}

impl Me {
    pub fn company(&self) -> Option<String> {
        self.org.clone().filter(|o| {
            !o.is_empty() && o != "personal" && !o.starts_with("me-") && !o.starts_with('_')
        })
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct LabLogin {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub org: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Record {
    pub id: String,
    #[serde(default)]
    pub cells: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Collection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub hint: String,
    #[serde(default)]
    pub columns: Vec<String>,
    #[serde(default)]
    pub records: Vec<Record>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct Wall {
    pub org: String,
    #[serde(default)]
    pub me: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default)]
    pub collections: Vec<Collection>,
}
