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
pub struct CommitTableRequest {
    #[serde(rename = "requirements")]
    pub requirements: Vec<crate::models::TableRequirement>,
    #[serde(rename = "updates")]
    pub updates: Vec<crate::models::TableUpdate>,
}

impl CommitTableRequest {
    pub fn new(requirements: Vec<crate::models::TableRequirement>, updates: Vec<crate::models::TableUpdate>) -> CommitTableRequest {
        CommitTableRequest {
            requirements,
            updates,
        }
    }
}


