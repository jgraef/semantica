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
    user::UserId,
};
use serde::Deserialize;
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

#[derive(Clone, Debug)]
pub struct Client {
    client: reqwest::Client,
    base_url: Url,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        let client = reqwest::Client::new();
        Self { client, base_url }
    }

    pub async fn register(&self, name: String) -> Result<NewUserResponse, Error> {
        let mut url = self.base_url.clone();
        {
            let mut path = url.path_segments_mut().unwrap();
            path.push("register");
        };

        let response = self
            .client
            .post(url)
            .json(&NewUserRequest { name })
            .send()
            .await?
            .into_api_result_json::<NewUserResponse>()
            .await?;
        Ok(response)
    }

    pub async fn login(
        &self,
        user_id: UserId,
        auth_secret: AuthSecret,
    ) -> Result<(), Error> {
        let mut url = self.base_url.clone();
        {
            let mut path = url.path_segments_mut().unwrap();
            path.push("login");
        };

        let _response = self
            .client
            .post(url)
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

    pub async fn logout(
        &self,
    ) -> Result<(), Error> {
        let mut url = self.base_url.clone();
        {
            let mut path = url.path_segments_mut().unwrap();
            path.push("logout");
        };

        let _response = self
            .client
            .get(url)
            .send()
            .await?
            .into_api_result()
            .await?;

        Ok(())
    }
}
