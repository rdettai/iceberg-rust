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
pub struct ReportMetricsRequest {
    #[serde(rename = "report-type")]
    pub report_type: String,
    #[serde(rename = "table-name")]
    pub table_name: String,
    #[serde(rename = "snapshot-id")]
    pub snapshot_id: i64,
    #[serde(rename = "filter")]
    pub filter: Box<crate::models::Expression>,
    #[serde(rename = "projection")]
    pub projection: Box<crate::models::Schema>,
    #[serde(rename = "metrics")]
    pub metrics: ::std::collections::HashMap<String, crate::models::MetricResult>,
}

impl ReportMetricsRequest {
    pub fn new(report_type: String, table_name: String, snapshot_id: i64, filter: crate::models::Expression, projection: crate::models::Schema, metrics: ::std::collections::HashMap<String, crate::models::MetricResult>) -> ReportMetricsRequest {
        ReportMetricsRequest {
            report_type,
            table_name,
            snapshot_id,
            filter: Box::new(filter),
            projection: Box::new(projection),
            metrics,
        }
    }
}


