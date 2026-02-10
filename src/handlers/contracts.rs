use actix_web::{HttpResponse, Responder, web};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::db::contracts as contract_db;
use crate::db::gigs as gig_db;
use crate::models::contracts::{CreateContract, Status, UpdateContractStatus};

/// POST /api/contracts — a client sends a contract request on a freelancer's gig.
///
/// The `user_id` is automatically set from the authenticated user's JWT (the client).
/// The gig must exist, the client cannot contract on their own gig, and only one
/// contract per client per gig is allowed.
pub async fn create_contract(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    body: web::Json<CreateContractRequest>,
) -> impl Responder {
    let client_id = user.0.id;
    let gig_id = body.gig_id;

    // 1. Verify the gig exists.
    let gig = match gig_db::get_gig_by_id(db.get_ref(), gig_id).await {
        Ok(Some(gig)) => gig,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Gig {gig_id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // 2. Prevent clients from contracting on their own gig.
    if gig.user_id == client_id {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "You cannot create a contract on your own gig",
        }));
    }

    // 3. Check for duplicate contract (one per client per gig).
    match contract_db::contract_exists_for_gig_and_user(db.get_ref(), gig_id, client_id).await {
        Ok(true) => {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "You have already sent a contract request for this gig",
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
        _ => {}
    }

    // 4. Create the contract.
    let input = CreateContract {
        gig_id,
        user_id: client_id,
    };

    match contract_db::insert_contract(db.get_ref(), input).await {
        Ok(contract) => HttpResponse::Created().json(contract),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to create contract: {e}"),
        })),
    }
}

/// GET /api/contracts — list contracts relevant to the authenticated user.
///
/// Returns contracts where the user is either:
/// - The client (user_id on the contract), OR
/// - The freelancer (owner of the gig referenced by the contract).
pub async fn get_contracts(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let user_id = user.0.id;

    // Get contracts where user is the client.
    let as_client = match contract_db::get_contracts_by_user_id(db.get_ref(), user_id).await {
        Ok(contracts) => contracts,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // Get all gigs owned by this user, then get contracts on those gigs.
    let user_gigs = match gig_db::get_gigs_by_user_id(db.get_ref(), user_id).await {
        Ok(gigs) => gigs,
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    let mut as_freelancer: Vec<crate::models::contracts::Model> = Vec::new();
    for gig in &user_gigs {
        match contract_db::get_contracts_by_gig_id(db.get_ref(), gig.id).await {
            Ok(contracts) => as_freelancer.extend(contracts),
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {e}"),
                }));
            }
        }
    }

    // Merge and deduplicate (a user could be both client and gig owner in theory,
    // though we prevent self-contracts).
    let mut all_contracts = as_client;
    for contract in as_freelancer {
        if !all_contracts.iter().any(|c| c.id == contract.id) {
            all_contracts.push(contract);
        }
    }

    HttpResponse::Ok().json(all_contracts)
}

/// GET /api/contracts/{id} — get a single contract.
///
/// Only the client (user_id on the contract) or the freelancer (gig owner) can view it.
pub async fn get_contract(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let contract_id = path.into_inner();
    let user_id = user.0.id;

    let contract = match contract_db::get_contract_by_id(db.get_ref(), contract_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Contract {contract_id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // Check authorization: user must be the client or the gig owner.
    if contract.user_id != user_id {
        match gig_db::get_gig_by_id(db.get_ref(), contract.gig_id).await {
            Ok(Some(gig)) if gig.user_id == user_id => {} // authorized as gig owner
            Ok(_) => {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "You can only view contracts you are involved in",
                }));
            }
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {e}"),
                }));
            }
        }
    }

    HttpResponse::Ok().json(contract)
}

/// PUT /api/contracts/{id}/status — freelancer (gig owner) accepts or rejects a contract.
///
/// Only the gig owner can update the status. The contract must be in Pending status.
pub async fn update_status(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateContractStatus>,
) -> impl Responder {
    let contract_id = path.into_inner();
    let user_id = user.0.id;

    // 1. Fetch the contract.
    let contract = match contract_db::get_contract_by_id(db.get_ref(), contract_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Contract {contract_id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // 2. Verify the authenticated user is the gig owner (freelancer).
    match gig_db::get_gig_by_id(db.get_ref(), contract.gig_id).await {
        Ok(Some(gig)) if gig.user_id == user_id => {} // authorized
        Ok(Some(_)) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only the gig owner (freelancer) can accept or reject contracts",
            }));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "The gig associated with this contract no longer exists",
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    }

    // 3. Only allow status updates on Pending contracts.
    if contract.status != Status::Pending {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!(
                "Contract is already {:?}. Only pending contracts can be updated.",
                contract.status
            ),
        }));
    }

    // 4. Update the status.
    match contract_db::update_contract_status(db.get_ref(), contract_id, body.into_inner()).await {
        Ok(updated) => HttpResponse::Ok().json(updated),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to update contract status: {e}"),
        })),
    }
}

/// DELETE /api/contracts/{id} — client withdraws a pending contract request.
///
/// Only the client who created the contract can withdraw it, and only while it is Pending.
pub async fn delete_contract(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let contract_id = path.into_inner();
    let user_id = user.0.id;

    // 1. Fetch the contract.
    let contract = match contract_db::get_contract_by_id(db.get_ref(), contract_id).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Contract {contract_id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    };

    // 2. Only the client who created the contract can withdraw it.
    if contract.user_id != user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only withdraw your own contract requests",
        }));
    }

    // 3. Only allow withdrawal of Pending contracts.
    if contract.status != Status::Pending {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": format!(
                "Contract is already {:?}. Only pending contracts can be withdrawn.",
                contract.status
            ),
        }));
    }

    // 4. Delete the contract.
    match contract_db::delete_contract(db.get_ref(), contract_id).await {
        Ok(result) => {
            if result.rows_affected > 0 {
                HttpResponse::Ok().json(serde_json::json!({
                    "message": format!("Contract {contract_id} withdrawn"),
                }))
            } else {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": format!("Contract {contract_id} not found"),
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to delete contract: {e}"),
        })),
    }
}

/// GET /api/contracts/gig/{gig_id} — get all contracts for a specific gig.
///
/// Only the gig owner (freelancer) can view all contracts on their gig.
pub async fn get_contracts_by_gig(
    user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let gig_id = path.into_inner();
    let user_id = user.0.id;

    // Verify the authenticated user owns the gig.
    match gig_db::get_gig_by_id(db.get_ref(), gig_id).await {
        Ok(Some(gig)) if gig.user_id == user_id => {} // authorized
        Ok(Some(_)) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Only the gig owner can view contracts for this gig",
            }));
        }
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Gig {gig_id} not found"),
            }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Database error: {e}"),
            }));
        }
    }

    match contract_db::get_contracts_by_gig_id(db.get_ref(), gig_id).await {
        Ok(contracts) => HttpResponse::Ok().json(contracts),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        })),
    }
}

/// GET /api/contracts/user/{user_id} — get all contracts sent by a specific user (client).
///
/// Users can only view their own sent contracts.
pub async fn get_contracts_by_user(
    auth_user: AuthenticatedUser,
    db: web::Data<DatabaseConnection>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let target_user_id = path.into_inner();

    // Users can only view their own contracts.
    if auth_user.0.id != target_user_id {
        return HttpResponse::Forbidden().json(serde_json::json!({
            "error": "You can only view your own contracts",
        }));
    }

    match contract_db::get_contracts_by_user_id(db.get_ref(), target_user_id).await {
        Ok(contracts) => HttpResponse::Ok().json(contracts),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Database error: {e}"),
        })),
    }
}

// ── Request DTOs ──

/// Request body for POST /api/contracts.
/// Only `gig_id` is required — `user_id` comes from the JWT.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateContractRequest {
    pub gig_id: Uuid,
}
