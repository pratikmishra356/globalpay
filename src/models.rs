use chrono::NaiveDateTime;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use serde::Serialize;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[diesel(treat_none_as_null = true)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub password: String,
    pub contact_number: Option<i64>,
    pub last_login: NaiveDateTime,
    pub token: String,
    pub current_balance: i64,
    pub age: Option<i32>,
    pub joined_date: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Clone)]
#[diesel(table_name = crate::schema::transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[diesel(treat_none_as_null = true)]
pub struct Transaction {
    pub id: String,
    pub type_: String,
    pub from_uid: Option<String>,
    pub to_uid: Option<String>,
    pub external_id: Option<String>,
    pub amount: i64,
    pub status: String,
    pub start_time: NaiveDateTime,
}

#[derive(Queryable, Selectable, Insertable, AsChangeset, Serialize, Clone)]
#[diesel(table_name = crate::schema::user_transaction)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
#[diesel(treat_none_as_null = true)]
pub struct UserTransaction {
    pub id: String,
    pub user_id: String,
    pub txn_id: String,
    pub amount: i64,
    pub type_: String,
    pub update_time: NaiveDateTime,
}
