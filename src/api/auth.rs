pub mod login;
pub mod oauth;

use oauth2::basic::{BasicErrorResponseType, BasicTokenType};
use oauth2::{
    Client, EmptyExtraTokenFields, EndpointNotSet, EndpointSet, ExtraTokenFields,
    RevocationErrorResponseType, StandardErrorResponse, StandardRevocableToken,
    StandardTokenIntrospectionResponse, StandardTokenResponse,
};
use serde::{Deserialize, Serialize};
use url::Url;

/* PUBLIC */

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub google_id: String,
    pub email: String,
}

#[allow(unused)]
#[derive(Clone, Deserialize)]
pub struct OauthRedirQueryParams {
    code: String,
    state: String,
    scope: String,
}

/* PRIVATE */

#[allow(unused)]
#[derive(Clone, Deserialize)]
struct GoogleIdToken {
    aud: String,
    exp: i64,
    iat: i64,
    iss: String,
    sub: String,
    amr: Option<Vec<String>>,
    auth_time: Option<i64>,
    at_hash: Option<String>,
    azp: Option<String>,
    email: Option<String>,
    email_verified: Option<bool>,
    given_name: Option<String>,
    hd: Option<String>,
    locale: Option<String>,
    name: Option<String>,
    nonce: Option<String>, // TODO implement
    picture: Option<Url>,
    profile: Option<Url>,
}

#[derive(Deserialize, Debug, Serialize)]
struct GoogleTokenExtraFields {
    pub id_token: Option<String>,
}

impl ExtraTokenFields for GoogleTokenExtraFields {}

type OauthSimpleClient = Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<GoogleTokenExtraFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub type AuthSession = axum_login::AuthSession<login::Backend>;
