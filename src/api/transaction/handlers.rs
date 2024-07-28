use super::types::InitiateTxn;
use crate::{
    models,
    schema::{
        transactions::{self},
        user_transaction, users,
        users::dsl::*,
    },
    types::{AppData, DbConnection, User as LoginUser},
};
use actix_web::{
    error, put,
    web::{Data, Json},
    HttpResponse, Scope,
};
use chrono::Utc;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, PooledConnection},
    ExpressionMethods,
};
use serde_json::json;
use uuid::Uuid;

pub fn endpoints() -> Scope {
    Scope::new("")
        .service(user_initiate_handler)
        .service(external_initiate_handler)
}

fn update_sender_transaction(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    sender_txn_struct: models::UserTransaction,
    txn_struct: &mut models::Transaction,
) -> Result<(), actix_web::error::Error> {
    let _ = conn
        .build_transaction()
        .read_write()
        .run(|conn| {
            let _ = users
                .filter(id.eq(sender_txn_struct.user_id.clone()))
                .filter(users::current_balance.ge(sender_txn_struct.amount.clone()))
                .select(id)
                .first::<String>(conn)?;
            let _ = diesel::insert_into(user_transaction::table)
                .values(sender_txn_struct.clone())
                .execute(conn)?;
            (*txn_struct) = models::Transaction {
                status: "inprogress".to_string(),
                ..(*txn_struct).clone()
            };
            let _ = diesel::update(transactions::table)
                .set(txn_struct.clone())
                .execute(conn)?;
            let _ = diesel::update(users::table)
                .set(current_balance.eq(current_balance - sender_txn_struct.amount.clone()))
                .execute(conn)?;
            Ok(())
        })
        .map_err(|err: diesel::result::Error| {
            log::error!("error while updating sender transaction {err}");
            error::ErrorInternalServerError("Something went wrong, money not debited")
        })?;
    Ok(())
}

fn update_receiver_transaction(
    conn: &mut PooledConnection<ConnectionManager<PgConnection>>,
    receiver_txn_struct: models::UserTransaction,
    txn_struct: &mut models::Transaction,
) -> Result<(), actix_web::error::Error> {
    let _ = conn
        .build_transaction()
        .read_write()
        .run(|conn| {
            let _ = diesel::insert_into(user_transaction::table)
                .values(receiver_txn_struct.clone())
                .execute(conn)?;
            (*txn_struct) = models::Transaction {
                status: "complete".to_string(),
                ..(*txn_struct).clone()
            };
            let _ = diesel::update(transactions::table)
                .set(txn_struct.clone())
                .execute(conn)?;
            let _ = diesel::update(users::table)
                .set(current_balance.eq(current_balance + receiver_txn_struct.amount.clone()))
                .execute(conn)?;
            Ok(())
        })
        .map_err(|err: diesel::result::Error| {
            log::error!("error while updating sender transaction {err}");
            error::ErrorInternalServerError("Something went wrong, money not debited")
        })?;
    Ok(())
}

#[put("/user/initiate")]
async fn user_initiate_handler(
    req_data: Json<InitiateTxn>,
    loggedin_user: LoginUser,
    db_conn: DbConnection,
) -> actix_web::Result<HttpResponse> {
    let DbConnection(mut conn) = db_conn;

    if loggedin_user.user_id != req_data.sender_id {
        return Ok(HttpResponse::BadRequest().json(json!({"message": "Invalid sender id"})));
    }
    if req_data.type_ == "internal" {
        let receiver_user = users
            .filter(id.eq(req_data.receiver_id.clone()))
            .select(models::User::as_select())
            .first(&mut conn);
        if let Err(diesel::result::Error::NotFound) = receiver_user {
            return Err(error::ErrorBadRequest(
                json!({"message": "Invalid receiver_id"}),
            ));
        }
    }

    let txn_id = Uuid::new_v4().to_string();
    let mut txn_struct = models::Transaction {
        id: txn_id.clone(),
        type_: req_data.type_.clone(),
        from_uid: Some(loggedin_user.user_id.clone()),
        to_uid: if req_data.type_ == "internal" {
            Some(req_data.receiver_id.clone())
        } else {
            None
        },
        external_id: if req_data.type_ == "external" {
            Some(req_data.receiver_id.clone())
        } else {
            None
        },
        amount: req_data.amount,
        status: "started".to_string(),
        start_time: Utc::now().naive_utc(),
    };

    let sender_txn_struct = models::UserTransaction {
        id: Uuid::new_v4().to_string(),
        user_id: loggedin_user.user_id.clone(),
        txn_id: txn_id.clone(),
        amount: req_data.amount.clone(),
        type_: "debited".to_string(),
        update_time: Utc::now().naive_utc(),
    };

    update_sender_transaction(&mut conn, sender_txn_struct, &mut txn_struct)?;
    let _ = diesel::update(transactions::table)
        .set(txn_struct.clone())
        .execute(&mut conn)
        .map_err(|_| error::ErrorInternalServerError("Something went wrong"))?;

    if req_data.type_ == "internal".to_string() {
        let receiver_txn_struct = models::UserTransaction {
            id: Uuid::new_v4().to_string(),
            user_id: req_data.receiver_id.clone(),
            txn_id: txn_id.clone(),
            amount: req_data.amount.clone(),
            type_: "credited".to_string(),
            update_time: Utc::now().naive_utc(),
        };

        update_receiver_transaction(&mut conn, receiver_txn_struct, &mut txn_struct)?;
    } else {
        txn_struct = models::Transaction {
            status: "completed".to_string(),
            ..txn_struct
        };
        let _ = diesel::update(transactions::table)
            .set(txn_struct.clone())
            .execute(&mut conn)
            .map_err(|_| error::ErrorInternalServerError("Something went wrong"))?;
    }

    Ok(HttpResponse::Ok().json(json!({"message": "Payment Successful"})))
}

#[put("/external/initiate")]
async fn external_initiate_handler(
    state: Data<AppData>,
    req_data: Json<InitiateTxn>,
) -> actix_web::Result<HttpResponse> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| error::ErrorInternalServerError("Something went wrongaa"))?;

    let user_data = users
        .filter(id.eq(req_data.receiver_id.clone()))
        .select(models::User::as_select())
        .first(&mut conn);

    let user = match user_data {
        Ok(val) => Ok(val),
        Err(diesel::result::Error::NotFound) => Err(error::ErrorBadRequest("User does not exist")),
        Err(err) => {
            log::error!("error while fetching user for external transaction {err}");
            Err(error::ErrorInternalServerError("Something went wrong"))
        }
    }?;

    let txn_id = Uuid::new_v4().to_string();
    let mut txn_struct = models::Transaction {
        id: txn_id.clone(),
        type_: req_data.type_.clone(),
        from_uid: Some(user.id.clone()),
        to_uid: Some(req_data.receiver_id.clone()),
        external_id: Some(req_data.sender_id.clone()),
        amount: req_data.amount,
        status: "started".to_string(),
        start_time: Utc::now().naive_utc(),
    };
    let _ = diesel::insert_into(transactions::table)
        .values(txn_struct.clone())
        .execute(&mut conn)
        .map_err(|err| {
            log::error!("error while creating transaction {err}");
            error::ErrorInternalServerError("Something went wrong, money not debited")
        })?;
    let receiver_txn_struct = models::UserTransaction {
        id: Uuid::new_v4().to_string(),
        user_id: user.id.clone(),
        txn_id: txn_id.clone(),
        amount: req_data.amount.clone(),
        type_: "credited".to_string(),
        update_time: Utc::now().naive_utc(),
    };

    update_receiver_transaction(&mut conn, receiver_txn_struct, &mut txn_struct)?;

    Ok(HttpResponse::Ok().json(json!({"message": "Payment Successful"})))
}
