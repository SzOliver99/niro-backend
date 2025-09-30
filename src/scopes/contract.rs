use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        contract::{Contract, ContractType, PaymentFrequency, PaymentMethod},
        customer::Customer,
        user::{User, UserRole},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn contract_scope() -> Scope {
    web::scope("/contract")
        .route("/create", web::post().to(create_contract))
        .route("/modify", web::put().to(modify_contract))
        .route(
            "/get-all/{user_uuid}",
            web::get().to(get_contracts_by_user_uuid),
        )
        .route("/{contract_uuid}", web::get().to(get_contract_by_uuid))
        .route(
            "/{contract_uuid}/customer",
            web::get().to(get_customer_uuid),
        )
        .route("/change/user", web::put().to(change_contract_handler))
        .route("/delete", web::delete().to(delete_contract))
}

#[derive(Deserialize, Clone)]
struct CustomerJson {
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
}
#[derive(Deserialize, Clone)]
struct CreateContractJson {
    customer: CustomerJson,
    contract_number: String,
    contract_type: ContractType,
    annual_fee: i32,
    payment_frequency: PaymentFrequency,
    payment_method: PaymentMethod,
    user_uuid: Uuid,
    created_by: String,
}
async fn create_contract(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<CreateContractJson>,
) -> impl Responder {
    let customer = Customer {
        full_name: Some(data.customer.full_name.clone()),
        phone_number: Some(data.customer.phone_number.clone()),
        email: Some(data.customer.email.clone()),
        address: Some(data.customer.address.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };
    let contract = Contract {
        contract_number: Some(data.contract_number.clone()),
        contract_type: Some(data.contract_type.clone()),
        annual_fee: Some(data.annual_fee.clone()),
        payment_frequency: Some(data.payment_frequency.clone()),
        payment_method: Some(data.payment_method.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match Contract::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        data.user_uuid,
        customer,
        contract,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Szerződés sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Clone)]
struct ModifyContractJson {
    contract_uuid: Uuid,
    contract_number: String,
    contract_type: ContractType,
    annual_fee: i32,
    payment_frequency: PaymentFrequency,
    payment_method: PaymentMethod,
}
async fn modify_contract(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<ModifyContractJson>,
) -> impl Responder {
    let contract = Contract {
        contract_number: Some(data.contract_number.clone()),
        contract_type: Some(data.contract_type.clone()),
        annual_fee: Some(data.annual_fee),
        payment_frequency: Some(data.payment_frequency.clone()),
        payment_method: Some(data.payment_method.clone()),
        ..Default::default()
    };

    match Contract::modify(&web_data.db, data.contract_uuid, contract).await {
        Ok(_) => HttpResponse::Created().json("Sikeresen megváltoztattad a szerződést!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_contracts_by_user_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Contract::get_all(&web_data.db, &web_data.key, user_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_contract_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    contract_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Contract::get_by_uuid(&web_data.db, contract_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_customer_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    contract_uuid: web::Path<Uuid>,
) -> impl Responder {
    match Contract::get_customer_uuid(&web_data.db, contract_uuid.into_inner()).await {
        Ok(customer_uuid) => HttpResponse::Ok().json(customer_uuid),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize)]
struct ChangeLeadsHandlerJson {
    user_full_name: String,
    contract_uuids: Vec<Uuid>,
}
async fn change_contract_handler(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeLeadsHandlerJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Contract::change_handler(
        &web_data.db,
        data.user_full_name.clone(),
        data.contract_uuids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Szerződésért felelős üzletkötő megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_contract(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<Uuid>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match Contract::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Szerződés(ek) sikeresen törölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
