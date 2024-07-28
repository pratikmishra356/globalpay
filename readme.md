# Local Setup

```docker compose build```

if you get libpq conflict error, then you need to clean docker cache
using this command ```docker builder prune``` -- this will clear all the build cache

```docker compose up```

base_url = http://localhost:8070

## Api Doc

## User 


path  -> /user/register

Method -> put
```
Req_body -> {
 name: String,
 email: String,
 password: String

}

Resp_body -> {
user_id: string,
token: string
}
```
path  -> /user/login

Method -> put
```
Req_body -> {
 email: String,
 password: String
}

Resp_body -> {
user_id: string,
token: string
}
```

path  -> /user/detail

Method -> get

header -> token : string
```
Resp_body -> {		 
id: string,
name: string,
email: string,
contact_number:integer,
current_balance: integer
}
```
path  -> /user/transaction/list

Method -> get

header -> token : string

query_params: limit, offset
```  
Resp_body -> {
[
id: string,
user_id: string,
txn_id: string,
amount: string,
type_: string,
update_time: timestamp
]
}

```

## Transaction

path  -> /transaction/user/initiate

Method -> put

header -> token : string
```
Req_body -> {
sender_id: string,
receiver_id: string,
type_: string,
amount: integer
}
   
Resp_body -> {
“Message” : “Payment Successful”
}
```


path  -> /transaction/external/initiate

Method -> put
```
Req_body -> {
sender_id: string,
receiver_id: string,
type_: string,
amount: integer
}
Resp_body -> {
“Message” : “Payment Successful”
}

```