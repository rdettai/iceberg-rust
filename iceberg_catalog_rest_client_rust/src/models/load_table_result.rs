/*
 * Apache Iceberg REST Catalog API
 *
 * Defines the specification for the first version of the REST Catalog API. Implementations should ideally support both Iceberg table specs v1 and v2, with priority given to v2.
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */

/// LoadTableResult : Result used when a table is successfully loaded.  The table metadata JSON is returned in the `metadata` field. The corresponding file location of table metadata should be returned in the `metadata-location` field, unless the metadata is not yet committed. For example, a create transaction may return metadata that is staged but not committed. Clients can check whether metadata has changed by comparing metadata locations after the table has been created.  The `config` map returns table-specific configuration for the table's resources, including its HTTP client and FileIO. For example, config may contain a specific FileIO implementation class for the table depending on its underlying storage.



#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct LoadTableResult {
    /// May be null if the table is staged as part of a transaction
    #[serde(rename = "metadata-location", skip_serializing_if = "Option::is_none")]
    pub metadata_location: Option<String>,
    #[serde(rename = "metadata")]
    pub metadata: Box<crate::models::TableMetadata>,
    #[serde(rename = "config", skip_serializing_if = "Option::is_none")]
    pub config: Option<::std::collections::HashMap<String, String>>,
}

impl LoadTableResult {
    /// Result used when a table is successfully loaded.  The table metadata JSON is returned in the `metadata` field. The corresponding file location of table metadata should be returned in the `metadata-location` field, unless the metadata is not yet committed. For example, a create transaction may return metadata that is staged but not committed. Clients can check whether metadata has changed by comparing metadata locations after the table has been created.  The `config` map returns table-specific configuration for the table's resources, including its HTTP client and FileIO. For example, config may contain a specific FileIO implementation class for the table depending on its underlying storage.
    pub fn new(metadata: crate::models::TableMetadata) -> LoadTableResult {
        LoadTableResult {
            metadata_location: None,
            metadata: Box::new(metadata),
            config: None,
        }
    }
}


