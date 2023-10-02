# Rust dependency graph for GitHub organizations

This 

## How to query the results

Show all nodes:
```neo4j
MATCH (n) OPTIONAL MATCH (n)-[r]-() RETURN n, r;
```

Find all users of a certain package:
```
MATCH (n:Package {name:'tracing'})<-[:Dependency]-(user)  RETURN n, user;
```

Find all users of a certain package and of a certain version:
```
MATCH (n:Package {name:'tracing'})<-[:Dependency{version:'0.1.129'}]-(user)  RETURN n, user;
```

Show the graph of all internal crates and their usage:
```
MATCH (n:Package)<-[:Dependency{registry:'truelayer-rustlayer'}]-(user)  RETURN n, user;
```