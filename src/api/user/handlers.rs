use super::types::{LoginRequest, RegisterRequest, UserResponse};
use crate::{
    models,
    schema::users::dsl::*,
    schema::{user_transaction, users},
};
use crate::{
    types::{DbConnection, User as loginUser},
    utils::generate_token,
};
use actix_web::{
    error, get, put,
    web::{self, Json},
    HttpResponse, Scope,
};
use chrono::Utc;
use diesel::{prelude::*, ExpressionMethods};

use serde_json::{from_value, json, Map, Value};
use uuid::Uuid;

pub fn endpoints() -> Scope {
    Scope::new("")
        .service(register)
        .service(login)
        .service(detail)
        .service(transaction_list)
}

#[put("/register")]
async fn register(
    req_data: Json<RegisterRequest>,
    db_conn: DbConnection,
) -> actix_web::Result<HttpResponse> {
    let DbConnection(mut conn) = db_conn;
    let user_id = Uuid::new_v4().to_string();
    let user_token = generate_token(user_id.clone())
        .map_err(|_| error::ErrorInternalServerError("Something went wrong"))?;
    if !(req_data.email.clone().contains("@gmail.com")) {
        return Err(error::ErrorBadRequest("Invalid email id"))
    }
    let hashed_token = blake3::hash(user_token.clone().as_bytes()).to_string();
    let hashed_pasword = blake3::hash(req_data.password.clone().as_bytes()).to_string();
    let user_struct = models::User {
        id: user_id.clone(),
        name: req_data.name.clone(),
        email: req_data.email.clone(),
        password: hashed_pasword.clone(),
        joined_date: Utc::now().naive_utc(),
        current_balance: 0,
        age: None,
        contact_number: None,
        last_login: Utc::now().naive_utc(),
        token: hashed_token.clone(),
    };

    let existing_user_id = users
        .filter(email.eq(req_data.email.clone()))
        .select(id)
        .first::<String>(&mut conn);
    match existing_user_id {
        Ok(val) => Ok(HttpResponse::BadRequest().json(json!({"message": val}))),
        Err(diesel::result::Error::NotFound) => {
            let _ = diesel::insert_into(users::table)
                .values(user_struct)
                .execute(&mut conn)
                .map_err(|_| error::ErrorInternalServerError("Something went wrong"))?;
            Ok(HttpResponse::Ok()
                .json(json!({"user_id": user_id.clone(), "token": user_token.clone()})))
        }
        Err(err) => {
            log::error!("error while registering user {err}");
            Ok(
                HttpResponse::InternalServerError()
                    .json(json!({"message": "Something went wrong"})),
            )
        }
    }
}

#[put("/login")]
async fn login(
    req_data: Json<LoginRequest>,
    db_conn: DbConnection,
) -> actix_web::Result<HttpResponse> {
    let DbConnection(mut conn) = db_conn;
    let hashed_pasword = blake3::hash(req_data.password.clone().as_bytes()).to_string();
    let user_data = users
        .filter(email.eq(req_data.email.clone()))
        .filter(password.eq(hashed_pasword.clone()))
        .select(models::User::as_select())
        .first(&mut conn);
    match user_data {
        Ok(mut val) => {
            let user_token = generate_token(val.id.clone())
                .map_err(|_| error::ErrorInternalServerError("Something went wrongss"))?;
            let hashed_token = blake3::hash(user_token.clone().as_bytes()).to_string();
            val = models::User {
                token: hashed_token.clone(),
                ..val
            };

            let _ = diesel::update(users::table)
                .set(val.clone())
                .execute(&mut conn)
                .map_err(|err| {
                    log::error!("error while updating user {err}");
                    error::ErrorInternalServerError("Something went wrongnnn")
                })?;
            Ok(HttpResponse::Ok()
                .json(json!({"user_id": val.id.clone(), "token": user_token.clone()})))
        }
        Err(diesel::result::Error::NotFound) => {
            Ok(HttpResponse::BadRequest().json(json!({"message": "Invalid username or password"})))
        }
        Err(err) => {
            log::error!("error while fetching user {err}");
            Ok(HttpResponse::InternalServerError()
                .json(json!({"message": "Something went wrongssss"})))
        }
    }
}

#[get("/detail")]
async fn detail(
    db_conn: DbConnection,
    loggedin_user: loginUser,
) -> actix_web::Result<HttpResponse> {
    let DbConnection(mut conn) = db_conn;

    let user_data = users
        .filter(token.eq(loggedin_user.user_id.clone()))
        .select(models::User::as_select())
        .first(&mut conn);

    match user_data {
        Ok(val) => {
            let user_resp = UserResponse {
                id: val.id,
                name: val.name,
                email: val.email,
                contact_number: val.contact_number,
                current_balance: val.current_balance,
            };
            Ok(HttpResponse::Ok().json(json!(user_resp)))
        }
        Err(diesel::result::Error::NotFound) => {
            Ok(HttpResponse::BadRequest().json(json!({"message": "Invalid token"})))
        }
        Err(err) => {
            log::error!("error while fetching user detail {err}");
            Ok(HttpResponse::InternalServerError()
                .json(json!({"message": "Something went wrongssss"})))
        }
    }
}

#[get("/transaction/list")]
async fn transaction_list(
    db_conn: DbConnection,
    query_params: web::Query<Map<String, Value>>,
    loggedin_user: loginUser,
) -> actix_web::Result<HttpResponse> {
    let DbConnection(mut conn) = db_conn;

    let limit_val = query_params
        .get("limit")
        .and_then(|val| from_value::<i64>(val.clone()).ok())
        .unwrap_or(10);
    let offset_val = query_params
        .get("offset")
        .and_then(|val| from_value::<i64>(val.clone()).ok())
        .unwrap_or(0);
    let user_txn_list = user_transaction::table
        .filter(user_transaction::user_id.eq(loggedin_user.user_id.clone()))
        .select(models::UserTransaction::as_select())
        .order(user_transaction::update_time.desc())
        .limit(limit_val)
        .offset(offset_val)
        .load(&mut conn);

    match user_txn_list {
        Ok(val) => Ok(HttpResponse::Ok().json(json!(val))),
        Err(diesel::result::Error::NotFound) => {
            Ok(HttpResponse::BadRequest().json(json!({"message": "Invalid token"})))
        }
        Err(err) => {
            log::error!("error while fetching transaction list {err}");
            Ok(HttpResponse::InternalServerError()
                .json(json!({"message": "Something went wrongssss"})))
        }
    }
}
