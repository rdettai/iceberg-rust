use std::io::Write;
use std::sync::Arc;
use std::{fs::File, time::Duration};

use datafusion::arrow::array::{Float64Array, RecordBatch};
use datafusion::execution::context::SessionContext;
use datafusion_iceberg::catalog::catalog::IcebergCatalog;
use iceberg_rest_catalog::apis::configuration::Configuration;
use iceberg_rest_catalog::catalog::RestCatalog;
use iceberg_rust::catalog::bucket::ObjectStoreBuilder;
use iceberg_rust::catalog::namespace::Namespace;
use iceberg_rust::catalog::Catalog;
use iceberg_rust::spec::partition::{PartitionField, PartitionSpec, Transform};
use iceberg_rust::spec::schema::Schema;
use iceberg_rust::spec::types::{PrimitiveType, StructField, StructType, Type};
use iceberg_rust::table::Table;
use iceberg_sql_catalog::SqlCatalog;
use object_store::aws::AmazonS3Builder;
use tempfile::TempDir;
use testcontainers::core::wait::HttpWaitStrategy;
use testcontainers::core::{CmdWaitFor, ContainerPort};
use testcontainers::{
    core::{wait::LogWaitStrategy, AccessMode, ExecCommand, Mount, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use testcontainers_modules::localstack::LocalStack;
use testcontainers_modules::postgres::Postgres;

fn configuration(host: &str, port: u16) -> Configuration {
    Configuration {
        base_path: format!("http://{}:{}", host, port),
        user_agent: None,
        client: reqwest::Client::new().into(),
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    }
}

#[tokio::test]
async fn integration_trino_rest() {
    let docker_host = "172.17.0.1";

    let localstack = LocalStack::default()
        .with_env_var("SERVICES", "s3")
        .with_env_var("AWS_ACCESS_KEY_ID", "user")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "password")
        .start()
        .await
        .unwrap();

    localstack
        .exec(ExecCommand::new(vec![
            "awslocal",
            "s3api",
            "create-bucket",
            "--bucket",
            "warehouse",
        ]))
        .await
        .unwrap();

    let localstack_host = localstack.get_host().await.unwrap();
    let localstack_port = localstack.get_host_port_ipv4(4566).await.unwrap();

    let rest = GenericImage::new("tabulario/iceberg-rest", "1.6.0")
        .with_wait_for(WaitFor::Log(LogWaitStrategy::stdout(
            "INFO  [org.eclipse.jetty.server.Server] - Started ",
        )))
        .with_env_var("AWS_REGION", "us-east-1")
        .with_env_var("AWS_ACCESS_KEY_ID", "user")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "password")
        .with_env_var("CATALOG_WAREHOUSE", "s3://warehouse/")
        .with_env_var("CATALOG_IO__IMPL", "org.apache.iceberg.aws.s3.S3FileIO")
        .with_env_var(
            "CATALOG_S3_ENDPOINT",
            format!("http://{}:{}", &docker_host, &localstack_port),
        )
        .start()
        .await
        .unwrap();

    let rest_host = rest.get_host().await.unwrap();
    let rest_port = rest.get_host_port_ipv4(8181).await.unwrap();

    let cwd = std::env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    let tmp_dir = TempDir::new().unwrap();
    let file_path = tmp_dir.path().join("iceberg-properties");
    let mut tmp_file = File::create(&file_path).unwrap();

    writeln!(tmp_file, "connector.name=iceberg").unwrap();
    writeln!(tmp_file, "iceberg.catalog.type=rest").unwrap();
    writeln!(
        tmp_file,
        "iceberg.rest-catalog.uri=http://{}:{}",
        docker_host, rest_port
    )
    .unwrap();
    writeln!(tmp_file, "iceberg.rest-catalog.warehouse=s3://warehouse/").unwrap();
    writeln!(tmp_file, "iceberg.file-format=PARQUET").unwrap();
    writeln!(
        tmp_file,
        "hive.s3.endpoint=http://{}:{}",
        docker_host, localstack_port
    )
    .unwrap();
    writeln!(tmp_file, "hive.s3.path-style-access=true").unwrap();
    writeln!(tmp_file, "hive.s3.aws-access-key=user").unwrap();
    writeln!(tmp_file, "hive.s3.aws-secret-key=password").unwrap();

    let catalog_mount = Mount::bind_mount(
        file_path.as_os_str().to_str().unwrap(),
        "/etc/trino/catalog/iceberg.properties",
    )
    .with_access_mode(AccessMode::ReadOnly);

    let sql_mount = Mount::bind_mount(cwd.clone() + "/tests/trino/trino.sql", "/tmp/trino.sql")
        .with_access_mode(AccessMode::ReadOnly);

    let trino = GenericImage::new("trinodb/trino", "latest")
        .with_wait_for(WaitFor::Http(
            HttpWaitStrategy::new("/v1/info")
                .with_port(ContainerPort::Tcp(8080))
                .with_response_matcher_async(|response| async move {
                    response.json::<serde_json::Value>().await.unwrap()["starting"] == false
                }),
        ))
        .with_env_var("AWS_REGION", "us-east-1")
        .with_env_var("AWS_ACCESS_KEY_ID", "user")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "password")
        .with_mount(catalog_mount)
        .with_mount(sql_mount)
        .with_startup_timeout(Duration::from_secs(180))
        .start()
        .await
        .unwrap();

    let trino_port = trino.get_host_port_ipv4(8080).await.unwrap();

    trino
        .exec(
            ExecCommand::new(vec![
                "trino",
                "--catalog",
                "iceberg",
                "--file",
                "/tmp/trino.sql",
                &format!("http://{}:{}", docker_host, trino_port),
            ])
            .with_cmd_ready_condition(CmdWaitFor::exit_code(0)),
        )
        .await
        .unwrap();

    let object_store = ObjectStoreBuilder::S3(
        AmazonS3Builder::new()
            .with_region("us-east-1")
            .with_allow_http(true)
            .with_access_key_id("user")
            .with_secret_access_key("password")
            .with_endpoint(format!("http://{}:{}", localstack_host, localstack_port)),
    );

    let catalog = Arc::new(RestCatalog::new(
        None,
        configuration(&rest_host.to_string(), rest_port),
        object_store,
    ));

    let tables = catalog
        .list_tabulars(&Namespace::try_new(&["test".to_owned()]).unwrap())
        .await
        .unwrap();

    assert_eq!(tables.len(), 8);

    let ctx = SessionContext::new();

    ctx.register_catalog(
        "iceberg",
        Arc::new(IcebergCatalog::new(catalog, None).await.unwrap()),
    );

    let df = ctx
        .sql("SELECT SUM(totalprice) FROM iceberg.test.orders;")
        .await
        .unwrap();

    // execute the plan
    let results: Vec<RecordBatch> = df.collect().await.expect("Failed to execute query plan.");

    let batch = results
        .into_iter()
        .find(|batch| batch.num_rows() > 0)
        .expect("All record batches are empty");

    let values = batch
        .column(0)
        .as_any()
        .downcast_ref::<Float64Array>()
        .expect("Failed to get values from batch.");

    assert!(values.value(0) - 2127396830.0 < 0.1);
}

#[tokio::test]
async fn integration_trino_sql() {
    let docker_host = "172.17.0.1";

    let localstack = LocalStack::default()
        .with_env_var("SERVICES", "s3")
        .with_env_var("AWS_ACCESS_KEY_ID", "user")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "password")
        .start()
        .await
        .unwrap();

    localstack
        .exec(ExecCommand::new(vec![
            "awslocal",
            "s3api",
            "create-bucket",
            "--bucket",
            "warehouse",
        ]))
        .await
        .unwrap();

    let localstack_host = localstack.get_host().await.unwrap();
    let localstack_port = localstack.get_host_port_ipv4(4566).await.unwrap();

    let postgres = Postgres::default()
        .with_db_name("postgres")
        .with_user("postgres")
        .with_password("postgres")
        .start()
        .await
        .unwrap();

    let postgres_host = postgres.get_host().await.unwrap();
    let postgres_port = postgres.get_host_port_ipv4(5432).await.unwrap();

    let cwd = std::env::current_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    let tmp_dir = TempDir::new().unwrap();
    let file_path = tmp_dir.path().join("iceberg-properties");
    let mut tmp_file = File::create(&file_path).unwrap();

    writeln!(tmp_file, "connector.name=iceberg").unwrap();
    writeln!(tmp_file, "iceberg.catalog.type=jdbc").unwrap();
    writeln!(tmp_file, "iceberg.jdbc-catalog.catalog-name=iceberg").unwrap();
    writeln!(
        tmp_file,
        "iceberg.jdbc-catalog.connection-url=jdbc:postgresql://{}:{}/postgres",
        docker_host, postgres_port
    )
    .unwrap();
    writeln!(
        tmp_file,
        "iceberg.jdbc-catalog.default-warehouse-dir=s3://warehouse/"
    )
    .unwrap();
    writeln!(
        tmp_file,
        "iceberg.jdbc-catalog.driver-class=org.postgresql.Driver"
    )
    .unwrap();
    writeln!(tmp_file, "iceberg.jdbc-catalog.connection-user=postgres").unwrap();
    writeln!(
        tmp_file,
        "iceberg.jdbc-catalog.connection-password=postgres"
    )
    .unwrap();
    writeln!(tmp_file, "iceberg.file-format=PARQUET").unwrap();
    writeln!(
        tmp_file,
        "hive.s3.endpoint=http://{}:{}",
        docker_host, localstack_port
    )
    .unwrap();
    writeln!(tmp_file, "hive.s3.path-style-access=true").unwrap();
    writeln!(tmp_file, "hive.s3.aws-access-key=user").unwrap();
    writeln!(tmp_file, "hive.s3.aws-secret-key=password").unwrap();
    writeln!(tmp_file, "hive.s3.ssl.enabled=false").unwrap();

    let catalog_mount = Mount::bind_mount(
        file_path.as_os_str().to_str().unwrap(),
        "/etc/trino/catalog/iceberg.properties",
    )
    .with_access_mode(AccessMode::ReadOnly);

    let sql_mount = Mount::bind_mount(cwd.clone() + "/tests/trino/trino.sql", "/tmp/trino.sql")
        .with_access_mode(AccessMode::ReadOnly);

    let trino = GenericImage::new("trinodb/trino", "latest")
        .with_wait_for(WaitFor::Http(
            HttpWaitStrategy::new("/v1/info")
                .with_port(ContainerPort::Tcp(8080))
                .with_response_matcher_async(|response| async move {
                    response.json::<serde_json::Value>().await.unwrap()["starting"] == false
                }),
        ))
        .with_env_var("AWS_REGION", "us-east-1")
        .with_env_var("AWS_ACCESS_KEY_ID", "user")
        .with_env_var("AWS_SECRET_ACCESS_KEY", "password")
        .with_mount(catalog_mount)
        .with_mount(sql_mount)
        .with_startup_timeout(Duration::from_secs(180))
        .start()
        .await
        .unwrap();

    let trino_port = trino.get_host_port_ipv4(8080).await.unwrap();

    let object_store = ObjectStoreBuilder::S3(
        AmazonS3Builder::new()
            .with_region("us-east-1")
            .with_allow_http(true)
            .with_access_key_id("user")
            .with_secret_access_key("password")
            .with_endpoint(format!("http://{}:{}", localstack_host, localstack_port)),
    );

    let catalog = Arc::new(
        SqlCatalog::new(
            &format!(
                "postgres://postgres:postgres@{}:{}/postgres",
                postgres_host, postgres_port
            ),
            "iceberg",
            Arc::new(
                object_store
                    .build(iceberg_rust::catalog::bucket::Bucket::S3("warehouse"))
                    .unwrap(),
            ),
        )
        .await
        .unwrap(),
    );

    let schema = Schema::builder()
        .with_fields(
            StructType::builder()
                .with_struct_field(StructField {
                    id: 1,
                    name: "id".to_string(),
                    required: true,
                    field_type: Type::Primitive(PrimitiveType::Long),
                    doc: None,
                })
                .with_struct_field(StructField {
                    id: 2,
                    name: "customer_id".to_string(),
                    required: true,
                    field_type: Type::Primitive(PrimitiveType::Long),
                    doc: None,
                })
                .with_struct_field(StructField {
                    id: 3,
                    name: "product_id".to_string(),
                    required: true,
                    field_type: Type::Primitive(PrimitiveType::Long),
                    doc: None,
                })
                .with_struct_field(StructField {
                    id: 4,
                    name: "date".to_string(),
                    required: true,
                    field_type: Type::Primitive(PrimitiveType::Date),
                    doc: None,
                })
                .with_struct_field(StructField {
                    id: 5,
                    name: "amount".to_string(),
                    required: true,
                    field_type: Type::Primitive(PrimitiveType::Int),
                    doc: None,
                })
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    let partition_spec = PartitionSpec::builder()
        .with_partition_field(PartitionField::new(4, 1000, "day", Transform::Day))
        .build()
        .expect("Failed to create partition spec");

    Table::builder()
        .with_name("orders")
        .with_location("s3://warehouse/orders")
        .with_schema(schema)
        .with_partition_spec(partition_spec)
        .build(&["test".to_owned()], catalog.clone())
        .await
        .expect("Failed to create table");

    let ctx = SessionContext::new();

    ctx.register_catalog(
        "iceberg",
        Arc::new(IcebergCatalog::new(catalog, None).await.unwrap()),
    );

    let _ = ctx
        .sql(
            "INSERT INTO iceberg.test.orders (id, customer_id, product_id, date, amount) VALUES
                (1, 1, 1, '2020-01-01', 1),
                (2, 2, 1, '2020-01-01', 1),
                (3, 3, 1, '2020-01-01', 3),
                (4, 1, 2, '2020-02-02', 1),
                (5, 1, 1, '2020-02-02', 2),
                (6, 3, 3, '2020-02-02', 3);",
        )
        .await
        .expect("Failed to create query plan for insert")
        .collect()
        .await;

    trino
        .exec(
            ExecCommand::new(vec![
                "trino",
                "--catalog",
                "iceberg",
                "--execute",
                "SELECT sum(amount) FROM iceberg.test.orders;",
                "--output-format",
                "NULL",
                &format!("http://{}:{}", docker_host, trino_port),
            ])
            .with_cmd_ready_condition(CmdWaitFor::exit_code(0)),
        )
        .await
        .unwrap()
        .stderr_to_vec()
        .await
        .unwrap();
}
