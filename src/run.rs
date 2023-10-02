use crate::{CargoToml, Dependency, GithubClient, Neo4j, Package};

pub async fn run(
    github: GithubClient,
    organization: String,
    graph: Neo4j,
) -> Result<(), anyhow::Error> {
    println!("Starting to retrieve the list of Cargo.toml");
    let all_tomls = github
        .get_all_toml_paths_in_organization(&organization)
        .await?;
    println!("Found '{}' Cargo.toml.", all_tomls.len());

    for toml in all_tomls {
        let cargo_toml_content = github
            .get_file_content(&organization, &toml.repo, &toml.path)
            .await?;

        let cargo_toml = match CargoToml::from_string(toml.repo.clone(), cargo_toml_content.clone())
        {
            Ok(cargo_toml) => cargo_toml,
            Err(error) => {
                println!(
                    "There was an error when parsing '{}/{}'. {}",
                    toml.repo.clone(),
                    toml.path.clone(),
                    error
                );
                continue;
            }
        };

        println!(
            "Processing '{}/{}'",
            cargo_toml.get_repository_name(),
            cargo_toml.get_name()
        );

        let package = Package {
            name: cargo_toml.get_name(),
            repository: cargo_toml.get_repository_name(),
        };
        package.store(graph.client()).await;

        for (dependency_name, dependency_version, registry) in cargo_toml.get_dependencies() {
            let dependency = Dependency::new(
                dependency_name,
                dependency_version,
                package.name.clone(),
                registry,
            );
            dependency.store(graph.client()).await;
        }
    }

    Ok(())
}
