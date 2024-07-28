// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (id) {
        #[max_length = 100]
        id -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        #[max_length = 100]
        from_uid -> Nullable<Varchar>,
        #[max_length = 100]
        to_uid -> Nullable<Varchar>,
        external_id -> Nullable<Varchar>,
        amount -> Int8,
        status -> Varchar,
        start_time -> Timestamp,
    }
}

diesel::table! {
    user_transaction (id) {
        #[max_length = 100]
        id -> Varchar,
        #[max_length = 100]
        user_id -> Varchar,
        #[max_length = 100]
        txn_id -> Varchar,
        amount -> Int8,
        #[sql_name = "type"]
        type_ -> Varchar,
        update_time -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        #[max_length = 100]
        id -> Varchar,
        #[max_length = 200]
        name -> Varchar,
        #[max_length = 200]
        email -> Varchar,
        #[max_length = 200]
        password -> Varchar,
        contact_number -> Nullable<Int8>,
        last_login -> Timestamp,
        token -> Text,
        current_balance -> Int8,
        age -> Nullable<Int4>,
        joined_date -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(transactions, user_transaction, users,);
