use anyhow::Result;
use aws_sdk_ecr::{
    Client,
    types::{ImageDetail, Repository},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ECRRepository {
    pub repository_name: String,
    pub repository_uri: String,
    pub registry_id: String,
    pub created_at: Option<DateTime<Utc>>,
    pub image_tag_mutability: String,
    pub image_scanning_configuration: bool,
    pub encryption_configuration: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ECRImage {
    pub image_tag: Option<String>,
    pub image_digest: String,
    pub image_pushed_at: Option<DateTime<Utc>>,
    pub image_size_in_bytes: Option<i64>,
    pub vulnerability_scan_summary: Option<String>,
}

impl From<Repository> for ECRRepository {
    fn from(repo: Repository) -> Self {
        Self {
            repository_name: repo.repository_name().unwrap_or("").to_string(),
            repository_uri: repo.repository_uri().unwrap_or("").to_string(),
            registry_id: repo.registry_id().unwrap_or("").to_string(),
            created_at: repo.created_at().map(|dt| -> DateTime<Utc> {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_else(Utc::now)
            }),
            image_tag_mutability: repo
                .image_tag_mutability()
                .map(|itm| format!("{:?}", itm))
                .unwrap_or_else(|| "MUTABLE".to_string()),
            image_scanning_configuration: repo
                .image_scanning_configuration()
                .map(|isc| isc.scan_on_push())
                .unwrap_or(false),
            encryption_configuration: repo
                .encryption_configuration()
                .map(|ec| format!("{:?}", ec.encryption_type()))
                .unwrap_or_else(|| "AES256".to_string()),
        }
    }
}

impl From<ImageDetail> for ECRImage {
    fn from(image: ImageDetail) -> Self {
        let image_tag = image.image_tags().first().map(|tag| tag.to_string());

        let vulnerability_summary = image.image_scan_findings_summary().map(|summary| {
            if let Some(counts) = summary.finding_severity_counts() {
                let total: i32 = counts.values().sum();
                if total > 0 {
                    format!("{} findings", total)
                } else {
                    "No vulnerabilities".to_string()
                }
            } else {
                "Scan pending".to_string()
            }
        });

        Self {
            image_tag,
            image_digest: image.image_digest().unwrap_or("").to_string(),
            image_pushed_at: image.image_pushed_at().map(|dt| -> DateTime<Utc> {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos()).unwrap_or_else(Utc::now)
            }),
            image_size_in_bytes: image.image_size_in_bytes(),
            vulnerability_scan_summary: vulnerability_summary,
        }
    }
}

pub struct ECRService {
    client: Client,
}

impl ECRService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn list_repositories(&self) -> Result<Vec<ECRRepository>> {
        let resp = self.client.describe_repositories().send().await?;

        let repositories = resp
            .repositories()
            .iter()
            .map(|repo| ECRRepository::from(repo.clone()))
            .collect();

        Ok(repositories)
    }

    pub async fn get_repository_images(&self, repository_name: &str) -> Result<Vec<ECRImage>> {
        let resp = self
            .client
            .describe_images()
            .repository_name(repository_name)
            .send()
            .await?;

        let images = resp
            .image_details()
            .iter()
            .map(|img| ECRImage::from(img.clone()))
            .collect();

        Ok(images)
    }
}
