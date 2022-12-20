/*
 * Apache Iceberg REST Catalog API
 *
 * Defines the specification for the first version of the REST Catalog API. Implementations should ideally support both Iceberg table specs v1 and v2, with priority given to v2.
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LoadNamespaceMetadata200Response {
    /// Reference to one or more levels of a namespace
    #[serde(rename = "namespace")]
    pub namespace: Vec<String>,
    /// Properties stored on the namespace, if supported by the server. If the server does not support namespace properties, it should return null for this field. If namespace properties are supported, but none are set, it should return an empty object.
    #[serde(rename = "properties", skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
}

impl LoadNamespaceMetadata200Response {
    pub fn new(namespace: Vec<String>) -> LoadNamespaceMetadata200Response {
        LoadNamespaceMetadata200Response {
            namespace,
            properties: None,
        }
    }
}


