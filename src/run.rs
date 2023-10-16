use crate::{CargoToml, Dependency, GithubClient, Neo4j, Package};

pub async fn run(
    github: GithubClient,
    graph: Neo4j,
    specific_package: &str,
) -> Result<(), anyhow::Error> {
    let all_tomls = get_tomls_from_github(github, &specific_package).await?;

    store_tomls(graph, all_tomls).await?;

    Ok(())
}

async fn get_tomls_from_github(
    github: GithubClient,
    specific_package: &str,
) -> Result<Vec<CargoToml>, anyhow::Error> {
    tracing::info!("[START] get_tomls_from_github");
    let all_tomls = github
        .get_all_toml_paths_in_organization(specific_package)
        .await?;
    tracing::info!("Found '{}' Cargo.toml from GitHub.", all_tomls.len());

    let mut parsed_tomls: Vec<CargoToml> = Vec::new();

    for toml in all_tomls {
        let cargo_toml_content = github.get_file_content(&toml.repo, &toml.path).await?;

        match CargoToml::from_string(toml.repo.clone(), cargo_toml_content.clone()) {
            Ok(cargo_toml) => parsed_tomls.push(cargo_toml),
            Err(error) => {
                tracing::error!(
                    "There was an error when parsing '{}/{}'. {}",
                    toml.repo.clone(),
                    toml.path.clone(),
                    error
                );
                continue;
            }
        };
    }

    tracing::info!("Parsed '{}' Cargo.toml.", parsed_tomls.len());
    tracing::info!("[END] get_tomls_from_github");
    Ok(parsed_tomls)
}

async fn store_tomls(graph: Neo4j, tomls: Vec<CargoToml>) -> Result<(), anyhow::Error> {
    tracing::info!("[START] store_tomls");
    for cargo_toml in tomls {
        tracing::debug!(
            "Storing '{}/{}'",
            cargo_toml.get_repository_name(),
            cargo_toml.get_name()
        );

        let package = Package {
            name: cargo_toml.get_name(),
            repository: cargo_toml.get_repository_name(),
        };
        let _ = package.store(graph.client()).await;

        for (dependency_name, dependency_version, registry) in cargo_toml.get_dependencies() {
            let dependency = Dependency::new(
                dependency_name,
                dependency_version,
                package.name.clone(),
                registry,
            );
            let _ = dependency.store(graph.client()).await;
        }
    }
    tracing::info!("[END] store_tomls");
    Ok(())
}
