/*
 * Apache Iceberg REST Catalog API
 *
 * Defines the specification for the first version of the REST Catalog API. Implementations should ideally support both Iceberg table specs v1 and v2, with priority given to v2.
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::apis::ResponseContent;
use super::{Error, configuration};


/// struct for typed errors of method [`get_token`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetTokenError {
    Status400(crate::models::GetToken400Response),
    Status401(crate::models::GetToken400Response),
    Status5XX(crate::models::GetToken400Response),
    UnknownValue(serde_json::Value),
}


/// Exchange credentials for a token using the OAuth2 client credentials flow or token exchange.  This endpoint is used for three purposes - 1. To exchange client credentials (client ID and secret) for an access token This uses the client credentials flow. 2. To exchange a client token and an identity token for a more specific access token This uses the token exchange flow. 3. To exchange an access token for one with the same claims and a refreshed expiration period This uses the token exchange flow.  For example, a catalog client may be configured with client credentials from the OAuth2 Authorization flow. This client would exchange its client ID and secret for an access token using the client credentials request with this endpoint (1). Subsequent requests would then use that access token.  Some clients may also handle sessions that have additional user context. These clients would use the token exchange flow to exchange a user token (the \"subject\" token) from the session for a more specific access token for that user, using the catalog's access token as the \"actor\" token (2). The user ID token is the \"subject\" token and can be any token type allowed by the OAuth2 token exchange flow, including a unsecured JWT token with a sub claim. This request should use the catalog's bearer token in the \"Authorization\" header.  Clients may also use the token exchange flow to refresh a token that is about to expire by sending a token exchange request (3). The request's \"subject\" token should be the expiring token. This request should use the subject token in the \"Authorization\" header.
pub async fn get_token(configuration: &configuration::Configuration, grant_type: Option<&str>, scope: Option<&str>, client_id: Option<&str>, client_secret: Option<&str>, requested_token_type: Option<crate::models::TokenType>, subject_token: Option<&str>, subject_token_type: Option<crate::models::TokenType>, actor_token: Option<&str>, actor_token_type: Option<crate::models::TokenType>) -> Result<crate::models::GetToken200Response, Error<GetTokenError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/oauth/tokens", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    if let Some(ref local_var_token) = local_var_configuration.oauth_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.to_owned());
    };
    let mut local_var_form_params = std::collections::HashMap::new();
    if let Some(local_var_param_value) = grant_type {
        local_var_form_params.insert("grant_type", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = scope {
        local_var_form_params.insert("scope", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = client_id {
        local_var_form_params.insert("client_id", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = client_secret {
        local_var_form_params.insert("client_secret", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = requested_token_type {
        local_var_form_params.insert("requested_token_type", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = subject_token {
        local_var_form_params.insert("subject_token", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = subject_token_type {
        local_var_form_params.insert("subject_token_type", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = actor_token {
        local_var_form_params.insert("actor_token", local_var_param_value.to_string());
    }
    if let Some(local_var_param_value) = actor_token_type {
        local_var_form_params.insert("actor_token_type", local_var_param_value.to_string());
    }
    local_var_req_builder = local_var_req_builder.form(&local_var_form_params);

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetTokenError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}

