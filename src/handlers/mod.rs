pub mod auth;
pub mod contracts;
pub mod gigs;
pub mod portfolio;
pub mod users;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    // ── Auth routes (protected by JWT via the AuthenticatedUser extractor) ──
    cfg.service(
        web::scope("/auth")
            .route("/me", web::get().to(auth::me))
            .route("/complete-profile", web::post().to(auth::complete_profile)),
    );

    // ── User routes (all protected — require valid JWT) ──
    cfg.service(
        web::resource("/users")
            .route(web::get().to(users::get_users)),
    );
    cfg.service(
        web::resource("/users/{id}")
            .route(web::get().to(users::get_user))
            .route(web::put().to(users::update_user))
            .route(web::delete().to(users::delete_user)),
    );

    // ── Portfolio routes (all protected — require valid JWT) ──
    cfg.service(
        web::resource("/portfolios")
            .route(web::get().to(portfolio::get_portfolios))
            .route(web::post().to(portfolio::create_portfolio)),
    );
    cfg.service(
        web::resource("/portfolios/{id}")
            .route(web::get().to(portfolio::get_portfolio))
            .route(web::put().to(portfolio::update_portfolio))
            .route(web::delete().to(portfolio::delete_portfolio)),
    );
    cfg.service(
        web::resource("/portfolios/freelancer/{freelancer_id}")
            .route(web::get().to(portfolio::get_portfolios_by_freelancer)),
    );

    // ── Gig routes (all protected — require valid JWT) ──
    cfg.service(
        web::scope("/gigs")
            .route("", web::get().to(gigs::get_gigs))
            .route("", web::post().to(gigs::create_gig))
            .route("/{id}", web::get().to(gigs::get_gig))
            .route("/{id}", web::put().to(gigs::update_gig))
            .route("/{id}", web::delete().to(gigs::delete_gig))
            .route("/user/{user_id}", web::get().to(gigs::get_gigs_by_user_id))
            .route("/user/{user_id}", web::delete().to(gigs::delete_all_gig_by_user_id)),
    );

    // ── Contract routes (all protected — require valid JWT) ──
    cfg.service(
        web::scope("/contracts")
            .route("", web::get().to(contracts::get_contracts))
            .route("", web::post().to(contracts::create_contract))
            .route("/{id}", web::get().to(contracts::get_contract))
            .route("/{id}", web::delete().to(contracts::delete_contract))
            .route("/{id}/status", web::put().to(contracts::update_status))
            .route("/gig/{gig_id}", web::get().to(contracts::get_contracts_by_gig))
            .route("/user/{user_id}", web::get().to(contracts::get_contracts_by_user)),
    );
    
}
