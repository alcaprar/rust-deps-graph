docker rm --force neo4j || true
docker run --name neo4j -d -p7474:7474 -p7687:7687 -e NEO4J_AUTH=neo4j/VeryLongPhrase1! neo4j:5.10