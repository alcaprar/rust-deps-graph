use neo4rs::{query, ConfigBuilder, Graph};

pub struct Neo4j {
    client: Graph,
}

impl Neo4j {
    pub async fn new(
        host: &str,
        port: &str,
        database: &str,
        username: &str,
        password: &str,
    ) -> Self {
        let config = ConfigBuilder::default()
            .uri(format!("{}:{}", host, port).as_str())
            .user(username)
            .password(password)
            .db(database)
            .build()
            .unwrap();
        let client = Graph::connect(config)
            .await
            .expect("Error when connecting to the graph");
        Self { client }
    }

    pub fn client(&self) -> &Graph {
        &self.client
    }
}

pub struct Package {
    pub name: String,
    pub repository: String,
}

impl Package {
    pub async fn store(&self, graph: &Graph) -> Result<(), anyhow::Error> {
        println!("Storing '{}|{}'", self.repository, self.name);
        let statement = query("CREATE (n:Package { name: $name, repository: $repository })")
            .param("name", self.name.clone())
            .param("repository", self.repository.clone());

        graph
            .run(statement)
            .await
            .expect("Error when creating the node");

        Ok(())
    }
}

#[derive(Debug)]
pub struct Dependency {
    name: String,
    version: String,
    major: String,
    minor: String,
    patch: String,
    used_by: String,
    registry: String,
}

impl Dependency {
    pub fn new(name: String, version: String, used_by: String, registry: String) -> Self {
        println!("version: {}", version);
        let version = version.replace("~", "");
        let version = version.replace("=", "");
        let version_splitted: Vec<&str> = version.split(".").collect();
        let major = version_splitted[0].to_string();
        let minor = if version_splitted.len() > 1 {
            version_splitted[1].to_string()
        } else {
            "*".to_string()
        };
        let patch = if version_splitted.len() > 2 {
            version_splitted[2].to_string()
        } else {
            "*".to_string()
        };
        Self {
            name,
            version,
            major,
            minor,
            patch,
            used_by,
            registry,
        }
    }

    pub async fn store(&self, graph: &Graph) -> Result<(), anyhow::Error> {
        let create_package_statement =
            query("MERGE (p:Package{name: $name })").param("name", self.name.clone());

        graph
            .run(create_package_statement)
            .await
            .expect("Error when creating the package");

        println!("{:?}", self);

        let create_relation_statement = query("match (a:Package), (b:Package) where a.name = $aname AND b.name = $bname create (a)-[r:Dependency{version: $version, major: $major, minor: $minor, patch: $patch, registry: $registry}]->(b)")
        .param("aname", self.used_by.clone())
        .param("bname", self.name.clone())
        .param("version", self.version.clone())
        .param("major", self.major.clone())
        .param("minor", self.minor.clone())
        .param("patch", self.patch.clone())
        .param("registry", self.registry.clone());

        graph
            .run(create_relation_statement)
            .await
            .expect("Error when creating the relationship");

        Ok(())
    }
}
