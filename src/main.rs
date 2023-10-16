use std::env;

use anyhow::Context;
use clap::Parser;
use rust_dependency_graph::{run, GithubClient, Neo4j};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    github_token: String,
    #[arg(short, long)]
    organization: String,
    #[arg(short, long)]
    package: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }
    env_logger::init();

    let args = Args::parse();
    let github_token = args.github_token;
    let organization = args.organization;
    let package = args.package;

    let github = GithubClient::new(github_token, organization)?;

    let graph = Neo4j::new("localhost", "7687", "neo4j", "neo4j", "VeryLongPhrase1!").await;

    // TODO check if neo4j is running
    graph
        .health_check()
        .await
        .context("Neo4j docker is not running.")?;

    run(github, graph, &package).await?;

    Ok(())
}
