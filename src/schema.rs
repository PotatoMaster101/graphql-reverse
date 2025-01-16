use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::error::Result;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Root {
    pub data: Option<Data>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    #[serde(rename = "__schema")]
    pub schema: Schema,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Schema {
    pub types: Vec<Type>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Type {
    pub name: String,
    pub kind: String,
    pub fields: Option<Vec<Field>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TypeRef {
    pub name: Option<String>,
    pub kind: String,
    #[serde(rename = "ofType")]
    pub of_type: Option<Box<TypeRef>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: TypeRef,
}

impl Root {
    /// Returns the root node from an introspection JSON string.
    #[inline]
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str::<Root>(json)?)
    }
}

impl Schema {
    /// Returns the object types expressed as a map.
    #[inline]
    pub fn get_type_map(&self) -> HashMap<String, &Type> {
        self.types.iter().map(|t| (t.name.clone(), t)).collect()
    }

    /// Returns a specific object types expressed as a map.
    #[inline]
    pub fn filter_type_map(&self, kind: &str) -> HashMap<String, &Type> {
        self.types.iter().filter(|t| t.name == kind).map(|t| (t.name.clone(), t)).collect()
    }
}

impl Type {
    #[inline]
    pub fn is_object(&self) -> bool {
        self.kind == "OBJECT"
    }

    pub fn get_field(&self, field_name: &str, containing: bool) -> Option<&Field> {
        if let Some(fields) = &self.fields {
            if containing {
                fields.iter().find(|f| f.name.contains(field_name))
            } else {
                fields.iter().find(|f| f.name == field_name)
            }
        } else {
            None
        }
    }

    /// Returns the fields expressed as a map.
    #[inline]
    pub fn get_field_map(&self) -> HashMap<String, &Field> {
        if let Some(fields) = &self.fields {
            fields.iter().map(|f| (f.name.clone(), f)).collect()
        } else {
            HashMap::new()
        }
    }

    /// Checks whether this type is a relay.
    pub fn is_relay(&self) -> bool {
        if let Some(fields) = &self.fields {
            let names: HashSet<_> = fields.iter().map(|f| f.name.clone()).collect();
            if self.name == "PageInfo" && names.contains("hasNextPage") && names.contains("hasPreviousPage") {
                return true;
            }
            if self.name.ends_with("Connection") && names.contains("edges") && names.contains("pageInfo") {
                return true;
            }
            if self.name.ends_with("Edge") && names.contains("cursor") && names.contains("node") {
                return true;
            }
            false
        } else {
            false
        }
    }
}

impl TypeRef {
    /// Returns the deepest type info.
    pub fn get_deepest(&self) -> Self {
        if let Some(of_type) = &self.of_type {
            of_type.get_deepest()
        } else {
            self.clone()
        }
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        self.kind == "OBJECT"
    }
}

impl Field {
    pub fn get_type_name(&self) -> String {
        let deep = self.field_type.get_deepest();
        deep.name.unwrap_or_else(|| panic!("Field {} doesn't have a type - invalid schema?", self.name))
    }
}
