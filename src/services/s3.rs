use crate::errors::Errors;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use base64::prelude::BASE64_STANDARD;
use base64::{Engine, prelude};

#[derive(Clone)]
pub struct S3Service {
    client: Client,
    bucket: String,
    host: String,
}
impl S3Service {
    pub fn new(host: String, key_id: String, key_secret: String, bucket: String) -> Self {
        let credentials = Credentials::new(key_id, key_secret, None, None, "cellar");

        let config = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("cellar")) //
            .endpoint_url(format!("https://{host}"))
            .credentials_provider(credentials)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(config);

        Self {
            client,
            bucket,
            host,
        }
    }

    pub async fn upload_file(
        &self,
        bytes: Vec<u8>,
        extension: &str,
        content_type: &str,
        folder: &str,
    ) -> Result<String, Errors> {
        let key = format!("{}/{}.{}", folder, uuid::Uuid::new_v4(), extension);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(bytes))
            .content_type(content_type)
            .acl(ObjectCannedAcl::PublicRead)
            .send()
            .await?;
        Ok(format!("https://{}.{}/{}", self.bucket, self.host, key))
    }

    pub async fn upload_base_image(&self, data_url: &str, folder: &str) -> Result<String, Errors> {
        let (header, base64_data) = data_url
            .split_once(";base64,")
            .ok_or_else(|| Errors::Error400BadRequest("Format url invalide".to_string()))?;

        let mime_type = header
            .strip_prefix("data:")
            .ok_or_else(|| Errors::Error400BadRequest("Format d'image invalide".to_string()))?;
        let extension = match mime_type {
            "image/jpeg" | "image/jpg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            _ => {
                return Err(Errors::Error400BadRequest(
                    "Format d'image non supporté".to_string(),
                ));
            }
        };
        let bytes = BASE64_STANDARD
            .decode(base64_data)
            .map_err(|_| Errors::Error400BadRequest("Base 64 Invalide".to_string()))?;
        Ok(self
            .upload_file(bytes, extension, mime_type, folder)
            .await?)
    }
}
