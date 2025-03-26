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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[tokio::test]
    async fn test_parse_ecs_metadata() {
        let json_data = r#"
        {
            "DockerId": "2969e5e20eda3af46d590cd7adfed899862bbcce424ae438a51a2a0b0edfcda0",
            "Image": "939885537497.dkr.ecr.us-east-1.amazonaws.com/streamer:latest-production",
            "Labels": {
                "com.amazonaws.ecs.cluster": "production",
                "com.amazonaws.ecs.container-name": "streamer",
                "com.amazonaws.ecs.task-arn": "arn:aws:ecs:us-east-1:939885537497:task/production/021447970bce4bd58069f1925cd87bc0",
                "com.amazonaws.ecs.task-definition-family": "streamer",
                "com.amazonaws.ecs.task-definition-version": "12"
            },
            "Limits": {"CPU": 2, "Memory": 0}
        }"#;

        let metadata: ECSContainerMetadataV4 = serde_json::from_str(json_data)
            .expect("Failed to deserialize ECSContainerMetadataV4 JSON");

        assert_eq!(metadata.docker_id, "2969e5e20eda3af46d590cd7adfed899862bbcce424ae438a51a2a0b0edfcda0");
        assert_eq!(metadata.image, "939885537497.dkr.ecr.us-east-1.amazonaws.com/streamer:latest-production");
        assert_eq!(metadata.labels.cluster, "production");
        assert_eq!(metadata.labels.container_name, "streamer");
        assert_eq!(metadata.labels.task_arn, "arn:aws:ecs:us-east-1:939885537497:task/production/021447970bce4bd58069f1925cd87bc0");
        assert_eq!(metadata.limits.cpu, 2);
        assert_eq!(metadata.limits.mem, 0);
    }
}