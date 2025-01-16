use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::{Display, Formatter};
use std::fs;
use clap::Parser;
use colored::{Color, ColoredString, Colorize};
use revql::schema::{Root, Type};

#[derive(Clone, Debug, Parser)]
struct Args {
    /// Path to JSON file containing the introspection.
    #[clap(required = true)]
    file: String,

    /// Type/field name to search for.
    #[clap(required = true)]
    search: String,

    /// Search name contains instead of exact match.
    #[clap(short, long)]
    containing: bool,

    /// Search for types only.
    #[clap(short, long = "type")]
    type_only: bool,

    /// Search for fields only.
    #[clap(short, long = "field")]
    field_only: bool,

    /// Shows relay types.
    #[clap(long = "show-relay")]
    show_relay: bool,
}

#[derive(Clone, Debug)]
struct TypeField {
    type_name: String,
    field_name: Option<String>,
}

impl Display for TypeField {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(field_name) = &self.field_name {
            write!(f, "{}.{}", self.type_name, field_name)
        } else {
            write!(f, "{}", self.type_name)
        }
    }
}

impl TypeField {
    fn new(type_name: &str, field_name: Option<String>) -> Self {
        Self {
            type_name: String::from(type_name),
            field_name,
        }
    }

    fn get_colored(&self, type_color: Color, field_color: Color) -> ColoredString {
        if let Some(field_name) = &self.field_name {
            format!("{}.{}", self.type_name.color(type_color), field_name.color(field_color)).into()
        } else {
            self.type_name.color(type_color)
        }
    }
}

fn search(
    start_type: &str,
    end_type: &str,
    type_map: &HashMap<String, &Type>,
) -> Vec<TypeField> {
    if start_type == end_type {
        return vec![TypeField::new(start_type, None)];
    }

    let mut visited: HashSet<_> = HashSet::from_iter([String::from(start_type)]);
    let mut queue = VecDeque::from_iter([String::from(start_type)]);
    let mut path: HashMap<String, Option<TypeField>> = HashMap::from([(String::from(start_type), None)]);
    while let Some(current) = queue.pop_front() {
        if !type_map.contains_key(&current) { continue; }
        if type_map[&current].name == end_type {
            let mut current_path = &path[end_type];
            let mut result = Vec::new();
            while let Some(current_type_field) = current_path {
                result.push(current_type_field.clone());
                current_path = &path[&current_type_field.type_name];
            }
            return result.iter().rev().cloned().collect();
        }

        let field_map = &type_map[&current].get_field_map();
        for (field_name, field) in field_map {
            let type_ref = field.field_type.get_deepest();
            if !type_ref.is_object() { continue; }
            if let Some(type_name) = type_ref.name {
                if visited.contains(&type_name) { continue; }
                visited.insert(type_name.clone());
                queue.push_back(type_name.clone());
                path.insert(type_name.clone(), Some(TypeField::new(&current, Some(field_name.clone()))));
            }
        }
    }
    Vec::new()
}

fn run_search(
    start_type: &str,
    end_type: &TypeField,
    type_map: &HashMap<String, &Type>
) -> Vec<Vec<TypeField>> {
    let mut result = Vec::new();
    if !type_map.contains_key(start_type) {
        return result;
    }

    let field_map = &type_map[start_type].get_field_map();
    for field in field_map.values() {
        let mut path = search(&field.get_type_name(), &end_type.type_name, type_map);
        if !path.is_empty() {
            path.insert(0, TypeField::new(start_type, Some(field.name.clone())));
            result.push(path);
        }
    }
    result
}

fn run_search_for_type(
    end_type: &str,
    containing: bool,
    show_relay: bool,
    type_map: &HashMap<String, &Type>
) {
    let end_types = if containing {
        type_map.iter().filter(|(name, _)| name.contains(end_type)).map(|(name, _)| TypeField::new(name, None)).collect()
    } else {
        Vec::from([TypeField::new(end_type, None)])
    };

    for end_type in end_types {
        if !type_map.contains_key(&end_type.type_name) || (!show_relay && type_map[&end_type.type_name].is_relay()) {
            continue;
        }

        for query in run_search("Query", &end_type, type_map) {
            print_path(&end_type, &query, show_relay, type_map);
        }

        for mutation in run_search("Mutation", &end_type, type_map) {
            print_path(&end_type, &mutation, show_relay, type_map);
        }
    }
}

fn run_search_for_field(
    end_field: &str,
    containing: bool,
    show_relay: bool,
    type_map: &HashMap<String, &Type>
) {
    for t in type_map.values() {
        if !type_map.contains_key(&t.name) || (!show_relay && type_map[&t.name].is_relay()) {
            continue;
        }

        let field = t.get_field(end_field, containing);
        if let Some(field) = field {
            let end_type = TypeField::new(&t.name, Some(field.name.clone()));
            for query in run_search("Query", &end_type, type_map) {
                print_path(&end_type, &query, show_relay, type_map);
            }

            for mutation in run_search("Mutation", &end_type, type_map) {
                print_path(&end_type, &mutation, show_relay, type_map);
            }
        }
    }
}

fn print_path(result: &TypeField, path: &[TypeField], show_relay: bool, type_map: &HashMap<String, &Type>) {
    print!("{}: ", result.get_colored(Color::Red, Color::Red));
    for idx in 0..path.len() {
        if show_relay || !type_map[&path[idx].type_name].is_relay() {
            if idx > 0 {
                print!(" -> ")
            }
            print!("{}", path[idx].get_colored(Color::Green, Color::White));
        }
    }
    println!();
}

fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.file).expect("Invalid file");
    let root = Root::from_json(&content).expect("Invalid schema");
    if root.data.is_none() {
        println!("Empty schema");
        return;
    }

    let data = root.data.unwrap();
    let type_map = data.schema.get_type_map();
    if args.type_only {
        run_search_for_type(&args.search, args.containing, args.show_relay, &type_map);
    } else if args.field_only {
        run_search_for_field(&args.search, args.containing, args.show_relay, &type_map);
    } else {
        run_search_for_type(&args.search, args.containing, args.show_relay, &type_map);
        run_search_for_field(&args.search, args.containing, args.show_relay, &type_map);
    }
}
