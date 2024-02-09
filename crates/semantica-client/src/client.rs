use std::{
    fmt::Display,
    sync::Arc,
};

use eventsource_stream::Eventsource;
use reqwest::{
    Response,
    StatusCode,
};
use semantica_protocol::{
    auth::{
        AuthRequest,
        AuthResponse,
        AuthSecret,
        NewUserRequest,
        NewUserResponse,
    },
    error::ApiError,
    node::{
        NodeId,
        NodeResponse,
        ResponseNode,
    },
    user::{
        InventoryResponse,
        UserId,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use url::Url;

use crate::{
    stream::EventStream,
    Error,
};

/// Helper trait to turn reqwest::Response into something useful, i.e. either a
/// parsed json value, a stream of parsed json values, or in case of an error, a
/// useful error with status code and parsed api error.
trait IntoApiResult: Sized {
    async fn into_api_result(self) -> Result<Response, Error>;

    async fn into_api_result_json<T: for<'de> Deserialize<'de>>(self) -> Result<T, Error> {
        Ok(self.into_api_result().await?.json().await?)
    }

    async fn into_api_result_stream<T: for<'de> Deserialize<'de>>(
        self,
    ) -> Result<EventStream<T>, Error> {
        let stream = self.into_api_result().await?.bytes_stream().eventsource();
        Ok(EventStream::new(stream))
    }
}

impl IntoApiResult for Response {
    async fn into_api_result(self) -> Result<Response, Error> {
        let status_code = self.status();
        if status_code == StatusCode::OK {
            Ok(self)
        }
        else {
            let api_error: ApiError = self.json().await.unwrap_or(ApiError::Unknown);
            Err(Error::Api {
                status_code,
                api_error,
            })
        }
    }
}

struct UrlBuilder {
    url: Url,
}

impl UrlBuilder {
    pub fn add(mut self, s: impl Display) -> Self {
        let mut segments = self.url.path_segments_mut().unwrap();
        segments.push(&s.to_string());
        drop(segments);
        self
    }

    pub fn build(self) -> Url {
        self.url
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    base_url: Arc<Url>,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            base_url: Arc::new(base_url),
        }
    }

    fn url(&self) -> UrlBuilder {
        UrlBuilder {
            url: Url::clone(&self.base_url),
        }
    }

    pub async fn register(&self, name: String) -> Result<NewUserResponse, Error> {
        let response = self
            .client
            .post(self.url().add("register").build())
            .json(&NewUserRequest { name })
            .send()
            .await?
            .into_api_result_json::<NewUserResponse>()
            .await?;
        Ok(response)
    }

    pub async fn login(&self, user_id: UserId, auth_secret: AuthSecret) -> Result<(), Error> {
        let _response = self
            .client
            .post(self.url().add("login").build())
            .json(&AuthRequest::Secret {
                user_id,
                auth_secret,
            })
            .send()
            .await?
            .into_api_result_json::<AuthResponse>()
            .await?;

        Ok(())
    }

    pub async fn logout(&self) -> Result<(), Error> {
        let _response = self
            .client
            .get(self.url().add("logout").build())
            .send()
            .await?
            .into_api_result()
            .await?;

        Ok(())
    }

    pub async fn inventory(&self) -> Result<InventoryResponse, Error> {
        let response = self
            .client
            .get(self.url().add("inventory").build())
            .send()
            .await?
            .into_api_result_json::<InventoryResponse>()
            .await?;
        Ok(response)
    }

    pub async fn node(&self, selector: NodeSelector) -> Result<ResponseNode, Error> {
        let mut url = self.url().add("node");
        match selector {
            NodeSelector::UserPosition => {
                url = url.add("current");
            }
            NodeSelector::Id(node_id) => {
                url = url.add(node_id);
            }
        }

        let response = self
            .client
            .get(url.build())
            .send()
            .await?
            .into_api_result_json::<NodeResponse>()
            .await?;

        Ok(response.node)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum NodeSelector {
    UserPosition,
    Id(NodeId),
}
