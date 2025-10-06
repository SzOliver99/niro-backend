use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use chrono::NaiveDateTime;
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
        .route(
            "/{contract_uuid}/state",
            web::put().to(change_first_payment_state),
        )
        .route("/change/user", web::put().to(change_contract_handler))
        .route("/delete", web::delete().to(delete_contract))
        .route(
            "/chart/portfolio/get-all",
            web::get().to(get_portfolio_chart),
        )
        .route(
            "/chart/portfolio/{user_uuid}",
            web::get().to(get_portfolio_chart_by_user_uuid),
        )
        .route(
            "/chart/weekly/get-all",
            web::post().to(get_weekly_production_chart),
        )
        .route(
            "/chart/weekly/{user_uuid}",
            web::post().to(get_weekly_production_chart_by_user_uuid),
        )
        .route(
            "/chart/monthly/value/get-all",
            web::post().to(get_monthly_production_value_chart),
        )
        .route(
            "/chart/monthly/value/{user_uuid}",
            web::post().to(get_monthly_production_value_chart_by_user_uuid),
        )
        .route(
            "/chart/monthly/production/get-all",
            web::post().to(get_monthly_production_chart),
        )
        .route(
            "/chart/monthly/production/{user_uuid}",
            web::post().to(get_monthly_production_chart_by_user_uuid),
        )
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

async fn change_first_payment_state(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    contract_uuid: web::Path<Uuid>,
    data: web::Json<bool>,
) -> impl Responder {
    match Contract::change_first_payment_state(&web_data.db, contract_uuid.into_inner(), data.0)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("Szerződés első díj befizetés módosítva!"),
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

#[derive(Deserialize)]
struct ContractChartJson {
    start_date: NaiveDateTime,
    end_date: NaiveDateTime,
}
async fn get_portfolio_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_portfolio_chart(&web_data.db).await {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_portfolio_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_portfolio_chart_by_user_uuid(&web_data.db, user_uuid.into_inner()).await {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_weekly_production_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ContractChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_weekly_production_chart(&web_data.db, data.start_date, data.end_date).await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_weekly_production_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
    data: web::Json<ContractChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_weekly_production_chart_by_user_uuid(
        &web_data.db,
        user_uuid.into_inner(),
        data.start_date,
        data.end_date,
    )
    .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_monthly_production_value_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ContractChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_monthly_production_value_chart(&web_data.db, data.start_date, data.end_date)
        .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_monthly_production_value_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
    data: web::Json<ContractChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_monthly_production_value_chart_by_user_uuid(
        &web_data.db,
        user_uuid.into_inner(),
        data.start_date,
        data.end_date,
    )
    .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_monthly_production_chart(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ContractChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_monthly_production_chart(&web_data.db, data.start_date, data.end_date).await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_monthly_production_chart_by_user_uuid(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
    data: web::Json<ContractChartJson>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Manager, auth_token.id as i32).await
    {
        return ApiError::from(e).error_response();
    }

    match Contract::get_monthly_production_chart_by_user_uuid(
        &web_data.db,
        user_uuid.into_inner(),
        data.start_date,
        data.end_date,
    )
    .await
    {
        Ok(chart) => HttpResponse::Ok().json(chart),
        Err(e) => ApiError::from(e).error_response(),
    }
}
