# RevQL
GraphQL object reverse lookup tool. The goal of this tool is to:
- Search for all queries/mutations to a specific object
- Search for all queries/mutations to any object containing a specific field

## Building
```
cargo build --release
```

## Usage
```
Usage: revql [OPTIONS] <FILE> <SEARCH>

Arguments:
  <FILE>    Path to JSON file containing the introspection
  <SEARCH>  Type/field name to search for

Options:
  -c, --containing  Search name contains instead of exact match
  -t, --type        Search for types only
  -f, --field       Search for fields only
      --show-relay  Shows relay types
  -h, --help        Print help
```

## Examples
Search for all queries/mutations to a type named `User`:
```
revql --type schema.json User
```

Search for all queries/mutations to any type with `User` in it:
```
revql --type --containing schema.json User
```

Search for all queries/mutations to any type with a field named `username`:
```
revql --field schema.json username
```

Search for all queries/mutations to any type or type containing field named `name`:
```
revql schema.json name
```

Search for all queries/mutations to a type named `User` and show all relay nodes:
```
revql --type --show-relay schema.json User
```
