use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_ecr::Client as ECRClient;
use aws_types::region::Region;

pub async fn create_ecr_client(
    profile: Option<String>,
    region: Option<String>,
) -> Result<ECRClient> {
    let mut config_loader = aws_config::defaults(BehaviorVersion::latest());

    // CLI flags have highest priority
    if let Some(profile) = profile {
        config_loader = config_loader.profile_name(profile);
    }

    if let Some(region) = region {
        config_loader = config_loader.region(Region::new(region));
    }

    let config = config_loader.load().await;
    let client = ECRClient::new(&config);
    Ok(client)
}
