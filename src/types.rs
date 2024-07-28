use crate::utils::decode_token;
use crate::{models, schema::users::dsl::*};
use actix_web::{error, web::Data, Error, FromRequest};
use derive_more::{Deref, DerefMut};
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use serde_json::json;
use std::future::{ready, Ready};

#[derive(Clone)]
pub struct AppData {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
}

impl FromRequest for AppData {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let app_data = req.app_data::<AppData>();

        let result = match app_data {
            Some(v) => Ok(v.clone()),
            None => Err(error::ErrorInternalServerError("app data not set")),
        };
        ready(result)
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
    pub user_id: String,
    pub email: String,
    pub token: String,
}

impl FromRequest for User {
    type Error = actix_web::error::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let app_state = match req.app_data::<Data<AppData>>() {
            Some(state) => state,
            None => {
                log::info!("Token-FromRequest: Unable to get app_data from request");
                return ready(Err(error::ErrorInternalServerError("")));
            }
        };

        let db_conn = match app_state.db_pool.get() {
            Ok(conn) => Ok(DbConnection(conn)),
            Err(err) => {
                log::info!("Unable to get db connection from pool, error: {err}");
                Err(error::ErrorInternalServerError("Something went wrong"))
            }
        };

        let user_token = req.headers().get("token").and_then(|val| val.to_str().ok());
        match (user_token, db_conn) {
            (Some(token_val), Ok(DbConnection(mut conn))) => {
                let hashed_token = blake3::hash(token_val.to_owned().as_bytes()).to_string();
                let user_data = users
                    .filter(token.eq(hashed_token))
                    .select(models::User::as_select())
                    .first(&mut conn);
                if let Ok(user_details) = user_data {
                    let token_match = decode_token(user_details.id.clone(), token_val.to_owned());

                    if let Err(_) = token_match {
                        return ready(Err(error::ErrorUnauthorized(
                            json!({"message":"token mismatched"}),
                        )));
                    }

                    ready(Ok(User {
                        name: user_details.name,
                        user_id: user_details.id,
                        email: user_details.email,
                        token: token_val.to_owned(),
                    }))
                } else {
                    log::error!("No user was found while validating token");
                    return ready(Err(error::ErrorUnauthorized(
                        json!({"message":"invalid token provided"}),
                    )));
                }
            }
            _ => {
                log::error!("No user was found while validating token");
                return ready(Err(error::ErrorUnauthorized(
                    json!({"message":"invalid token provided"}),
                )));
            }
        }
    }
}

pub type PgDBConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Deref, DerefMut)]
pub struct DbConnection(pub PgDBConnection);
impl FromRequest for DbConnection {
    type Error = Error;
    type Future = Ready<Result<DbConnection, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let app_state = match req.app_data::<Data<AppData>>() {
            Some(state) => state,
            None => {
                log::info!("DbConnection-FromRequest: Unable to get app_data from request");
                return ready(Err(error::ErrorInternalServerError("")));
            }
        };
        let result = match app_state.db_pool.get() {
            Ok(conn) => Ok(DbConnection(conn)),
            Err(err) => {
                log::info!("Unable to get db connection from pool, error: {err}");
                Err(error::ErrorInternalServerError("Something went wrong"))
            }
        };
        ready(result)
    }
}
