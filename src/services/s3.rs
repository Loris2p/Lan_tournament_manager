use crate::errors::Errors;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::{BehaviorVersion, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::ObjectCannedAcl;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;

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
        Ok(format!("https://{}/{}/{}", self.host, self.bucket, key))
    }
    fn detect_image_type(bytes: &[u8]) -> Result<(&'static str, &'static str), Errors> {
        if bytes.len() < 12 {
            return Err(Errors::Error400BadRequest(
                "Fichier non compatible détecté".to_string(),
            ));
        } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Ok(("png", "image/png"));
        } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(("jpeg", "image/jpeg"));
        } else if &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            return Ok(("webp", "image/webp"));
        }
        Err(Errors::Error400BadRequest(
            "Fichier non compatible détecté".to_string(),
        ))
    }
    pub async fn upload_base_image(&self, data_url: &str, folder: &str) -> Result<String, Errors> {
        let (header, base64_data) = data_url
            .split_once(";base64,")
            .ok_or_else(|| Errors::Error400BadRequest("Format url invalide".to_string()))?;

        let declared_mime = header
            .strip_prefix("data:")
            .ok_or_else(|| Errors::Error400BadRequest("Format d'image invalide".to_string()))?;

        let bytes = BASE64_STANDARD
            .decode(base64_data)
            .map_err(|_| Errors::Error400BadRequest("Base 64 Invalide".to_string()))?;
        let (detected_ext, detected_mime) = Self::detect_image_type(&bytes)?;
        let is_matching = match (declared_mime, detected_mime) {
            ("image/png", "image/png") => true,
            ("image/jpeg" | "image/jpg", "image/jpeg") => true,
            ("image/webp", "image/webp") => true,
            _ => false,
        };

        if !is_matching {
            return Err(Errors::Error400BadRequest(
                "Le contenu binaire du fichier ne correspond pas au format déclaré".to_string(),
            ));
        }

        Ok(self
            .upload_file(bytes, detected_ext, detected_mime, folder)
            .await?)
    }
    pub async fn delete_folder(&self, folder: &str) -> Result<(), Errors> {
        let prefix = format!("{}/", folder.trim_end_matches('/'));
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self.client.list_objects_v2()
                .bucket(&self.bucket)
                .prefix(&prefix);

            if let Some(ref token) = continuation_token {
                req = req.continuation_token(token);
            }

            let resp = req.send().await?;

            if let Some(contents) = resp.contents {
                for obj in contents {
                    if let Some(key) = obj.key {
                        let _ = self.client
                            .delete_object()
                            .bucket(&self.bucket)
                            .key(key)
                            .send()
                            .await;
                    }
                }
            }

            if resp.is_truncated.unwrap_or(false) {
                continuation_token = resp.next_continuation_token;
            } else {
                break;
            }
        }
        Ok(())
    }

}
