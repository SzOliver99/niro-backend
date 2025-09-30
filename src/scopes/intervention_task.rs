use actix_web::{HttpResponse, Responder, ResponseError, Scope, web};
use chrono::NaiveDateTime;
use serde::Deserialize;
use uuid::Uuid;

use crate::models::intervention_task::{InterventionTask, InterventionTaskStatus};
use crate::{
    extractors::authentication_token::AuthenticationToken,
    models::{
        customer::Customer,
        user::{User, UserRole},
    },
    utils::error::ApiError,
    web_data::WebData,
};

pub fn intervention_task_scope() -> Scope {
    web::scope("/intervention-task")
        .route(
            "/create/{customer_uuid}",
            web::post().to(create_intervention_task),
        )
        .route(
            "/modify/{intervention_task_uuid}",
            web::put().to(modify_intervention_task),
        )
        .route(
            "/get-all/{user_uuid}",
            web::get().to(get_intervention_tasks_by_user_uuid),
        )
        .route(
            "/{intervention_task_uuid}",
            web::get().to(get_intervention_task_by_uuid),
        )
        .route(
            "/{intervention_task_uuid}/customer",
            web::get().to(get_customer_uuid),
        )
        .route(
            "/change/user",
            web::put().to(change_intervention_task_handler),
        )
        .route("/delete", web::delete().to(delete_intervention_task))
}

#[derive(Deserialize, Clone, Debug)]
struct CustomerJson {
    full_name: String,
    phone_number: String,
    address: String,
    email: String,
}
#[derive(Deserialize, Clone, Debug)]
struct InterventionTaskJson {
    contract_number: String,
    product_name: String,
    outstanding_days: i32,
    balance: i32,
    processing_deadline: NaiveDateTime,
    comment: String,
    status: InterventionTaskStatus,
}
#[derive(Deserialize, Clone)]
struct CreateInterventionTaskJson {
    customer: CustomerJson,
    intervention_task: InterventionTaskJson,
    created_by: String,
}
async fn create_intervention_task(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<CreateInterventionTaskJson>,
    customer_uuid: web::Path<Uuid>,
) -> impl Responder {
    let customer = Customer {
        full_name: Some(data.customer.full_name.clone()),
        phone_number: Some(data.customer.phone_number.clone()),
        email: Some(data.customer.email.clone()),
        address: Some(data.customer.address.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };
    let intervention_task = InterventionTask {
        contract_number: Some(data.intervention_task.contract_number.clone()),
        product_name: Some(data.intervention_task.product_name.clone()),
        outstanding_days: Some(data.intervention_task.outstanding_days),
        balance: Some(data.intervention_task.balance),
        processing_deadline: Some(data.intervention_task.processing_deadline),
        comment: Some(data.intervention_task.comment.clone()),
        status: Some(data.intervention_task.status.clone()),
        created_by: Some(data.created_by.clone()),
        ..Default::default()
    };

    match InterventionTask::create(
        &web_data.db,
        &web_data.key,
        &web_data.hmac_secret,
        customer_uuid.into_inner(),
        customer,
        intervention_task,
    )
    .await
    {
        Ok(_) => HttpResponse::Created().json("Intervenciós feladat sikeresen létrehozva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn modify_intervention_task(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    data: web::Json<InterventionTaskJson>,
    intervention_task_uuid: web::Path<Uuid>,
) -> impl Responder {
    let intervention_task = InterventionTask {
        contract_number: Some(data.contract_number.clone()),
        product_name: Some(data.product_name.clone()),
        outstanding_days: Some(data.outstanding_days),
        balance: Some(data.balance),
        processing_deadline: Some(data.processing_deadline),
        comment: Some(data.comment.clone()),
        status: Some(data.status.clone()),
        ..Default::default()
    };

    match InterventionTask::modify(
        &web_data.db,
        intervention_task_uuid.into_inner(),
        intervention_task,
    )
    .await
    {
        Ok(_) => {
            HttpResponse::Created().json("Sikeresen megváltoztattad az intervenciós feladatot!")
        }
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_intervention_tasks_by_user_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    user_uuid: web::Path<Uuid>,
) -> impl Responder {
    match InterventionTask::get_all(&web_data.db, &web_data.key, user_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_intervention_task_by_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    intervention_task_uuid: web::Path<Uuid>,
) -> impl Responder {
    match InterventionTask::get_by_uuid(&web_data.db, intervention_task_uuid.into_inner()).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn get_customer_uuid(
    web_data: web::Data<WebData>,
    _: AuthenticationToken,
    intervention_task_uuid: web::Path<Uuid>,
) -> impl Responder {
    match InterventionTask::get_customer_uuid(&web_data.db, intervention_task_uuid.into_inner())
        .await
    {
        Ok(customer_uuid) => HttpResponse::Ok().json(customer_uuid),
        Err(e) => ApiError::from(e).error_response(),
    }
}

#[derive(Deserialize, Debug)]
struct ChangeInterventionTasksHandlerJson {
    user_full_name: String,
    intervention_task_uuids: Vec<Uuid>,
}
async fn change_intervention_task_handler(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<ChangeInterventionTasksHandlerJson>,
) -> impl Responder {
    println!("{data:?}");
    if let Err(e) = User::require_role(&web_data.db, UserRole::Leader, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match InterventionTask::change_handler(
        &web_data.db,
        data.user_full_name.clone(),
        data.intervention_task_uuids.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Created()
            .json("Intervenciós feladat(ok)ért felelős üzletkötő megváltoztatva!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}

async fn delete_intervention_task(
    web_data: web::Data<WebData>,
    auth_token: AuthenticationToken,
    data: web::Json<Vec<Uuid>>,
) -> impl Responder {
    if let Err(e) = User::require_role(&web_data.db, UserRole::Agent, auth_token.id as i32).await {
        return ApiError::from(e).error_response();
    }

    match InterventionTask::delete(&web_data.db, data.0).await {
        Ok(_) => HttpResponse::Created().json("Intervenciós feladat(ok) sikeresen törölve!"),
        Err(e) => ApiError::from(e).error_response(),
    }
}
