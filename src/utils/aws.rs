use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_ecr::Client as ECRClient;

pub async fn create_ecr_client() -> Result<ECRClient> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = ECRClient::new(&config);
    Ok(client)
}

pub fn get_current_region() -> String {
    std::env::var("AWS_REGION")
        .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
        .unwrap_or_else(|_| "us-east-1".to_string())
}

pub fn get_current_profile() -> String {
    std::env::var("AWS_PROFILE").unwrap_or_else(|_| "default".to_string())
}
