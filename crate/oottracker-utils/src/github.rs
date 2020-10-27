use {
    std::future::Future,
    reqwest::{
        Body,
        Client,
        StatusCode,
    },
    semver::{
        SemVerError,
        Version,
    },
    serde::Deserialize,
    serde_json::json,
};

#[derive(Deserialize)]
pub(crate) struct Release {
    id: u64,
    tag_name: String,
    pub(crate) upload_url: String,
}

impl Release {
    pub(crate) fn version(&self) -> Result<Version, SemVerError> {
        self.tag_name[1..].parse()
    }
}

/// A GitHub repository. Provides API methods.
pub(crate) struct Repo {
    /// The GitHub user or organization who owns this repo.
    pub(crate) user: String,
    /// The name of the repo.
    pub(crate) name: String,
}

impl Repo {
    pub(crate) fn new(user: impl ToString, name: impl ToString) -> Repo {
        Repo {
            user: user.to_string(),
            name: name.to_string(),
        }
    }

    pub(crate) async fn latest_release(&self, client: &Client) -> reqwest::Result<Option<Release>> {
        let response = client.get(&format!("https://api.github.com/repos/{}/{}/releases/latest", self.user, self.name))
            .send().await?;
        if response.status() == StatusCode::NOT_FOUND { return Ok(None); } // no releases yet
        Ok(Some(
            response.error_for_status()?
                .json::<Release>().await?
        ))
    }

    /// Creates a draft release, which can be published using `Repo::publish_release`.
    pub(crate) async fn create_release(&self, client: &Client, name: String, tag_name: String, body: String) -> reqwest::Result<Release> {
        Ok(
            client.post(&format!("https://api.github.com/repos/{}/{}/releases", self.user, self.name))
                .json(&json!({
                    "body": body,
                    "draft": true,
                    "name": name,
                    "tag_name": tag_name
                }))
                .send().await?
                .error_for_status()?
                .json::<Release>().await?
        )
    }

    pub(crate) async fn publish_release(&self, client: &Client, release: Release) -> reqwest::Result<Release> {
        Ok(
            client.patch(&format!("https://api.github.com/repos/{}/{}/releases/{}", self.user, self.name, release.id))
                .json(&json!({"draft": false}))
                .send().await?
                .error_for_status()?
                .json::<Release>().await?
        )
    }

    pub(crate) fn release_attach<'a>(&self, client: &'a Client, release: &'a Release, name: &'a str, content_type: &'static str, body: impl Into<Body> + 'a) -> impl Future<Output = reqwest::Result<()>> + 'a {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static(content_type));
        async move {
            client.post(&release.upload_url.replace("{?name,label}", ""))
                .query(&[("name", name)])
                .headers(headers)
                .body(body)
                .send().await?
                .error_for_status()?;
            Ok(())
        }
    }
}
