use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use std::{thread, time};

pub struct GithubClient {
    octocrab: Octocrab,
    token: String,
    organization: String,
}

impl GithubClient {
    pub fn new(token: String, organization: String) -> Result<Self, anyhow::Error> {
        let octocrab = Octocrab::builder().personal_token(token.clone()).build()?;

        Ok(Self {
            octocrab,
            token,
            organization,
        })
    }

    pub async fn get_all_toml_paths_in_organization(
        &self,
        specific_package: &str,
    ) -> Result<Vec<CargoTomlPath>, anyhow::Error> {
        let mut page: u32 = 1;
        let per_page = 50;
        let mut has_next = true;

        let query = format!(
            "[dependencies] {} filename:Cargo.toml org:{}",
            specific_package, self.organization
        );

        let mut vector_result: Vec<CargoTomlPath> = Vec::new();

        while has_next {
            tracing::debug!("page '{}'", page);
            let results = self
                .octocrab
                .search()
                .code(&query)
                .page(page)
                .per_page(per_page)
                .send()
                .await
                .map_err(|err| {
                    tracing::error!("Error when calling github api '{}'", err);
                    err
                })?;

            for result in results.clone().into_iter() {
                vector_result.push(CargoTomlPath {
                    organization: self.organization.to_string(),
                    repo: result.repository.name,
                    path: result.path,
                })
            }

            if results.next.is_some() {
                page += 1;
                thread::sleep(time::Duration::from_secs(10));
            } else {
                has_next = false;
            }

            has_next = false;
        }
        Ok(vector_result)
    }

    pub async fn get_file_content(&self, repo: &str, path: &str) -> Result<String, anyhow::Error> {
        let file_url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            self.organization, repo, path
        );

        let client = reqwest::Client::new();
        let result: GetPathContentResponse = client
            .get(file_url)
            .header("Authorization", format!("token {}", self.token))
            .header("User-Agent", "App")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .json()
            .await?;

        let result = client
            .get(result.download_url)
            .header("User-Agent", "App")
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?
            .text()
            .await?;

        Ok(result)
    }
}

pub struct CargoTomlPath {
    pub organization: String,
    pub repo: String,
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetPathContentResponse {
    download_url: String,
}
