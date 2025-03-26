use serde::Deserialize;
use std::env;
use crate::error::ECSMetadataError;

const ECS_METADATA_V4_ENV_VAR: &str = "ECS_CONTAINER_METADATA_URI_V4";

// Initial information set (there is more available to extend it, format can be found at
// https://docs.aws.amazon.com/AmazonECS/latest/developerguide/task-metadata-endpoint-v4-response.html
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ECSContainerMetadataV4 {
    docker_id: String,
    image: String,
    labels: ECSContainerLabels,
    limits: ECSContainerLimits,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ECSContainerLabels {
    #[serde(rename = "com.amazonaws.ecs.cluster")]
    cluster: String,
    #[serde(rename = "com.amazonaws.ecs.container-name")]
    container_name: String,
    #[serde(rename = "com.amazonaws.ecs.task-arn")]
    task_arn: String,
    #[serde(rename = "com.amazonaws.ecs.task-definition-family")]
    task_definition_family: String,
    #[serde(rename = "com.amazonaws.ecs.task-definition-version")]
    task_definition_version: String,
}

#[derive(Deserialize, Debug)]
pub struct ECSContainerLimits {
    #[serde(rename = "CPU")]
    pub cpu: u16,
    #[serde(rename = "Memory")]
    pub mem: u16,
}

pub struct ECSMetadata {
    metadata: ECSContainerMetadataV4,
}

impl ECSMetadata {
    /// Initialize ECS metadata by fetching it from the AWS ECS metadata endpoint
    pub async fn init() -> Result<Self, ECSMetadataError> {
        let metadata_url = env::var(ECS_METADATA_V4_ENV_VAR)
            .map_err(|_| ECSMetadataError::EnvVarNotSet(ECS_METADATA_V4_ENV_VAR.to_string()))?;

        let response = reqwest::get(&metadata_url)
            .await?
            .error_for_status()?; // bail if not successful

        let metadata: ECSContainerMetadataV4 = response.json().await?;

        Ok(Self { metadata })
    }

    pub fn task_arn(&self) -> &str {
        &self.metadata.labels.task_arn
    }

    /// The ECS task ID is last portion of the ARN
    pub fn task_id(&self) -> Option<String> {
        self.metadata.labels.task_arn.split('/').last().map(ToString::to_string)
    }

    /// ECS cluster name
    pub fn cluster(&self) -> &str {
        &self.metadata.labels.cluster
    }

    /// CPU & Memory resource limits
    pub fn limits(&self) -> &ECSContainerLimits {
        &self.metadata.limits
    }

    pub fn docker_id(&self) -> &str {
        &self.metadata.docker_id
    }

    pub fn image(&self) -> &str {
        &self.metadata.image
    }

    pub fn task_definition_family(&self) -> &str {
        &self.metadata.labels.task_definition_family
    }

    pub fn task_definition_revision(&self) -> &str {
        &self.metadata.labels.task_definition_version
    }

    pub fn container_name(&self) -> &str {
        &self.metadata.labels.container_name
    }
}