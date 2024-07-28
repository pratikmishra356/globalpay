-- Your SQL goes here
CREATE TABLE users (
    id varchar(100) PRIMARY KEY,
    name varchar(200) not null,
    email varchar(200) not null,
    password varchar(200) not null,
    contact_number bigint,
    last_login timestamp not null,
    token text not null,
    current_balance bigint not null default(0),
    age int,
    joined_date timestamp not null
);


CREATE TABLE transactions (
    id varchar(100) PRIMARY KEY,
    type varchar not null,
    from_uid varchar(100),
    to_uid varchar(100),
    external_id varchar null,
    amount bigint not null,
    status varchar not null,
    start_time timestamp not null
);

CREATE TABLE user_transaction (
    id varchar(100) PRIMARY KEY,
    user_id varchar(100) not null,
    txn_id varchar(100) not null,
    amount bigint not null,
    type varchar not null,
    update_time timestamp not null,
    UNIQUE(user_id, txn_id)
);