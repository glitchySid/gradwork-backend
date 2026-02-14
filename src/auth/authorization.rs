use actix_web::HttpResponse;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::db::contracts as contract_db;
use crate::db::gigs as gig_db;
use crate::models::contracts::{Model, Status};

pub async fn verify_contract_party(
    db: &DatabaseConnection,
    contract_id: Uuid,
    user_id: Uuid,
) -> Result<Model, HttpResponse> {
    let contract = contract_db::get_contract_by_id(db, contract_id)
        .await
        .map_err(|e| {
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }))
        })?
        .ok_or_else(|| {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Contract {contract_id} not found"),
            }))
        })?;

    if contract.status != Status::Accepted {
        return Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Chat is only available for accepted contracts",
        })));
    }

    let is_client = contract.user_id == user_id;

    let is_freelancer = match gig_db::get_gig_by_id(db, contract.gig_id).await {
        Ok(Some(gig)) => gig.user_id == user_id,
        _ => false,
    };

    if !is_client && !is_freelancer {
        return Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You are not a party to this contract",
        })));
    }

    Ok(contract)
}

pub async fn verify_gig_owner(
    db: &DatabaseConnection,
    gig_id: Uuid,
    user_id: Uuid,
) -> Result<(), HttpResponse> {
    match gig_db::get_gig_by_id(db, gig_id).await {
        Ok(Some(gig)) if gig.user_id == user_id => Ok(()),
        Ok(Some(_)) => Err(HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You do not own this gig",
        }))),
        Ok(None) => Err(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Gig {gig_id} not found"),
        }))),
        Err(e) => Err(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        }))),
    }
}
