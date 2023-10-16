extern crate toml_edit;
use std::hash::{Hash, Hasher};
use std::{collections::HashSet, fs};
use toml_edit::{Document, Item, Value};

#[derive(Debug, Clone)]
pub struct CargoToml {
    raw_content: String,
    repository_name: String,
    name: String,
    dependencies: Vec<(String, String, String)>,
    versions_used: HashSet<String>,
}

impl Hash for CargoToml {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.repository_name.hash(state);
        self.name.hash(state);
    }
}

impl CargoToml {
    pub fn from_file(repository_name: String, file_path: &str) -> Result<Self, anyhow::Error> {
        let raw_content = fs::read_to_string(file_path)
            .map_err(|_| anyhow::anyhow!("Error when reading content from file"))?;

        Self::from_string(repository_name, raw_content)
    }

    pub fn from_string(
        repository_name: String,
        raw_content: String,
    ) -> Result<Self, anyhow::Error> {
        let parsed = raw_content
            .parse::<Document>()
            .map_err(|_error| anyhow::anyhow!("Error when parsing the toml."))?;

        let name = match Self::get_name_from_item(&parsed["package"]) {
            Some(name) => name.trim().replace('\"', ""),
            None => return Err(anyhow::anyhow!("name not found in the package")),
        };
        let dependencies = Self::get_dependencies_from_item(&parsed["dependencies"]);

        Ok(Self {
            raw_content,
            name,
            dependencies,
            versions_used: HashSet::new(),
            repository_name,
        })
    }

    pub fn get_id(&self) -> String {
        format!(
            "{}_{}",
            self.get_name().replace('-', "_"),
            self.get_repository_name().replace('-', "_")
        )
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_repository_name(&self) -> String {
        self.repository_name.clone()
    }

    pub fn get_dependencies(&self) -> Vec<(String, String, String)> {
        self.dependencies.clone()
    }

    pub fn get_raw_content(&self) -> String {
        self.raw_content.clone()
    }

    pub fn get_versions(&self) -> HashSet<String> {
        self.versions_used.clone()
    }

    pub fn add_version(&mut self, version: String) {
        self.versions_used.insert(version);
    }

    fn get_dependencies_from_item(dependencies_item: &Item) -> Vec<(String, String, String)> {
        let mut result: Vec<(String, String, String)> = Vec::new();
        let dependencies = match dependencies_item.as_table() {
            Some(table) => table,
            None => return result,
        };

        for dep in dependencies.iter() {
            let dep_name = dep.0;
            tracing::debug!("get_dependencies_from_item dep.1: {:?}", dep.1);
            let version = match Self::get_version_from_item(dep.1) {
                Some(version) => version.trim().replace('\"', ""),
                None => continue,
            };
            let registry = match Self::get_registry_from_item(dep.1) {
                Some(registry) => registry.trim().replace('\"', ""),
                None => "".to_string(),
            };

            result.push((dep_name.to_string(), version, registry))
        }

        result
    }

    fn get_name_from_item(package_item: &Item) -> Option<String> {
        match package_item.as_table() {
            Some(table) => match table.get("name") {
                Some(item) => item.as_value().map(|value| value.to_string()),
                None => None,
            },
            None => None,
        }
    }

    fn get_version_from_item(item: &Item) -> Option<String> {
        match item {
            Item::Value(value) => match value {
                Value::String(value) => Some(value.to_string()),
                Value::InlineTable(table) => {
                    table.get("version").map(|version| version.to_string())
                }
                Value::Integer(_)
                | Value::Float(_)
                | Value::Boolean(_)
                | Value::Datetime(_)
                | Value::Array(_) => None,
            },
            Item::None | Item::Table(_) | Item::ArrayOfTables(_) => None,
        }
    }

    fn get_registry_from_item(item: &Item) -> Option<String> {
        match item {
            Item::Value(value) => match value {
                Value::InlineTable(table) => {
                    table.get("registry").map(|registry| registry.to_string())
                }
                Value::String(_)
                | Value::Integer(_)
                | Value::Float(_)
                | Value::Boolean(_)
                | Value::Datetime(_)
                | Value::Array(_) => None,
            },
            Item::None | Item::Table(_) | Item::ArrayOfTables(_) => None,
        }
    }
}
