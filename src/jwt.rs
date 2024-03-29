//! defines the middleware for the views that require authentication.
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::config::GetConfigVariable;

#[cfg(feature = "actix")]
use futures::future::{Ready, ok, err};

#[cfg(feature = "actix")]
use actix_web::{
    dev::Payload,
    Error,
    FromRequest,
    HttpRequest,
    error::ErrorUnauthorized
};
use crate::errors::{
    NanoServiceError,
    NanoServiceErrorStatus
};


/// The attributes extracted from the auth token hiding in the header.
///
/// # Fields
/// * `user_id`: the ID of the user who's token it belongs to
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenBody {
    pub user_id: i32
}


/// JWT for authentication for an API request.
///
/// # Fields
/// * `user_id`: the ID of the user who's token it belongs to
/// * `handle`: the handle of the user who's token it belongs to
#[derive(Debug, Serialize, Deserialize)]
pub struct JwToken<X: GetConfigVariable> {
    pub user_id: i32,
    pub handle: Option<X>
}


impl <X: GetConfigVariable>JwToken<X> {

    /// Gets the secret key from the environment for encoding and decoding tokens.
    ///
    /// # Returns
    /// the key from the environment
    pub fn get_key() -> Result<String, NanoServiceError> {
        let key = <X>::get_config_variable("SECRET_KEY".to_string())?;
        return Ok(key)
    }

    /// Encodes the struct into a token.
    ///
    /// # Returns
    /// encoded token with fields of the current struct
    pub fn encode(self) -> Result<String, NanoServiceError> {
        let key = EncodingKey::from_secret(JwToken::<X>::get_key()?.as_ref());

        let body = TokenBody {
            user_id: self.user_id
        };
        return match encode(&Header::default(), &body, &key) {
            Ok(token) => Ok(token),
            Err(error) => Err(
                NanoServiceError::new(
                    error.to_string(),
                    NanoServiceErrorStatus::Unauthorized
                )
            )
        };
    }

    /// Decodes the token into a struct.
    ///
    /// # Arguments
    /// * `token` - The token to be decoded.
    ///
    /// # Returns
    /// decoded token with fields of the current struct
    pub fn decode(token: &str) -> Result<TokenBody, NanoServiceError> {
        let key = DecodingKey::from_secret(JwToken::<X>::get_key()?.as_ref());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.remove("exp");

        match decode::<TokenBody>(token, &key, &validation) {
            Ok(token_data) => return Ok(token_data.claims),
            Err(error) => return Err(
                NanoServiceError::new(
                    error.to_string(),
                    NanoServiceErrorStatus::Unauthorized
                )
            )
        };
    }

}


#[cfg(feature = "actix")]
impl<X: GetConfigVariable> FromRequest for JwToken<X> {
    type Error = Error;
    type Future = Ready<Result<JwToken<X>, Error>>;

    /// This gets fired when the JwToken is attached to a request. It fires before the request hits the view.
    /// # Arguments
    /// The arguments are needed in order for the impl of FromRequest to work.
    ///
    /// * req (&HttpRequest): the request that the token is going to be extracted from
    /// * _ (Payload): the payload stream (not used in this function but is needed)
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {

        match req.headers().get("token") {
            Some(data) => {
                let raw_token = data.to_str().unwrap().to_string();
                let token_result = JwToken::<X>::decode(&raw_token.as_str());

                match token_result {
                    Ok(token) => {
                        let jwt = JwToken::<X> {
                            user_id: token.user_id,
                            handle: None
                        };
                        return ok(jwt)
                    },
                    Err(error) => {
                        if error.message == "ExpiredSignature".to_owned() {
                            return err(ErrorUnauthorized("token expired"))
                        }
                        return err(ErrorUnauthorized("token can't be decoded"))
                    }
                }
            },
            None => {
                return err(ErrorUnauthorized("token not in header under key 'token'"))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[cfg(feature = "actix")]
    use serde_json::json;

    #[cfg(feature = "actix")]
    use actix_web::{
        HttpRequest,
        HttpResponse,
        App,
        web,
        http::header::ContentType,
        self,
        test::{
            TestRequest,
            init_service,
            call_service
        },
    };
    use crate::errors::NanoServiceError;


    struct FakeConfig;

    impl GetConfigVariable for FakeConfig {

        fn get_config_variable(variable: String) -> Result<String, NanoServiceError> {
            match variable.as_str() {
                "SECRET_KEY" => Ok("secret".to_string()),
                _ => Ok("".to_string())
            }
        }

    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ResponseFromTest {
        pub user_id: i32,
    }

    #[cfg(feature = "actix")]
    async fn pass_handle(token: JwToken<FakeConfig>, _: HttpRequest) -> HttpResponse {
        return HttpResponse::Ok().json(json!({"user_id": token.user_id}))
    }

    #[test]
    fn test_encode_decode() {
        let expected_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxfQ.J_RIIkoOLNXtd5IZcEwaBDGKGA3VnnYmuXnmhsmDEOs";
        let jwt = JwToken {
            user_id: 1,
            handle: Some(FakeConfig)
        };
        let encoded_token = jwt.encode().unwrap();
        assert_eq!(encoded_token, expected_token);
    }

    #[test]
    fn test_decode_token() {
        let expected_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxfQ.J_RIIkoOLNXtd5IZcEwaBDGKGA3VnnYmuXnmhsmDEOs";
        let decoded_token = JwToken::<FakeConfig>::decode(expected_token).unwrap();
        assert_eq!(decoded_token.user_id, 1);
    }

    #[cfg(feature = "actix")]
    #[actix_web::test]
    async fn test_no_token_request() {

        let app = init_service(App::new().route("/", web::get().to(pass_handle))).await;
        let req = TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();

        let resp = call_service(&app, req).await;
        assert_eq!("401", resp.status().as_str());
    }

    #[cfg(feature = "actix")]
    #[actix_web::test]
    async fn test_pass_check() {

        let app = init_service(App::new().route("/", web::get().to(pass_handle))).await;
        let req = TestRequest::default()
            .insert_header(ContentType::plaintext())
            .insert_header(("token", "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoxfQ.J_RIIkoOLNXtd5IZcEwaBDGKGA3VnnYmuXnmhsmDEOs"))
            .to_request();

        let resp = call_service(&app, req).await;
        assert_eq!("200", resp.status().as_str());
    }

}
