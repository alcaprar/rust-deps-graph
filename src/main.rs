use clap::Parser;
use rust_dependency_graph::{run, GithubClient, Neo4j};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    github_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let github_token = args.github_token;

    let github = GithubClient::new(github_token)?;
    let organization = "TrueLayer".to_string();

    let graph = Neo4j::new("localhost", "7687", "neo4j", "neo4j", "VeryLongPhrase1!").await;

    run(github, organization, graph).await?;

    Ok(())
}
