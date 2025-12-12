use crate::common::{ extract_json_response };
use iron_control_api::routes::tokens::{ PaginatedTokensResponse };
use axum::{ Router, routing::{ post, get }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

async fn create_test_router() -> ( Router, crate::common::test_state::TestAppState ) {
    let app_state = crate::common::test_state::TestAppState::new().await;
    let router = Router::new()
        .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
        .route( "/api/v1/api-tokens", get( iron_control_api::routes::tokens::list_tokens ) )
        .with_state( app_state.clone() );
    ( router, app_state )
}

fn generate_jwt_for_user( app_state: &crate::common::test_state::TestAppState, user_id: &str ) -> String {
    app_state.auth.jwt_secret
        .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
        .expect( "Failed to generate JWT" )
}

#[tokio::test]
async fn test_list_tokens_pagination() {
    let (router, app_state) = create_test_router().await;
    let jwt = generate_jwt_for_user(&app_state, "user_pag");

    // Create 3 tokens
    for i in 0..3 {
        let body = json!({ "user_id": "user_pag", "name": format!("token_{}", i) });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/api-tokens")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", jwt))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();
        router.clone().oneshot(req).await.unwrap();
    }

    // List with per_page=2
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/api-tokens?page=1&per_page=2")
        .header("authorization", format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();
    
    let res = router.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    
    let (_, body): (StatusCode, PaginatedTokensResponse) = extract_json_response(res).await;
    assert_eq!(body.data.len(), 2);
    assert_eq!(body.pagination.total, 3);
    assert_eq!(body.pagination.page, 1);
    assert_eq!(body.pagination.per_page, 2);
    assert_eq!(body.pagination.total_pages, 2);

    // Page 2
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/api-tokens?page=2&per_page=2")
        .header("authorization", format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();
    
    let res = router.oneshot(req).await.unwrap();
    let (_, body): (StatusCode, PaginatedTokensResponse) = extract_json_response(res).await;
    assert_eq!(body.data.len(), 1);
}

#[tokio::test]
async fn test_list_tokens_filtering() {
    let (router, app_state) = create_test_router().await;
    let jwt = generate_jwt_for_user(&app_state, "user_filter");

    // Create token with project
    let body1 = json!({ "user_id": "user_filter", "name": "t1", "project_id": "proj1" });
    let req1 = Request::builder()
        .method("POST")
        .uri("/api/v1/api-tokens")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", jwt))
        .body(Body::from(serde_json::to_string(&body1).unwrap()))
        .unwrap();
    router.clone().oneshot(req1).await.unwrap();

    // Create token without project
    let body2 = json!({ "user_id": "user_filter", "name": "t2" });
    let req2 = Request::builder()
        .method("POST")
        .uri("/api/v1/api-tokens")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", jwt))
        .body(Body::from(serde_json::to_string(&body2).unwrap()))
        .unwrap();
    router.clone().oneshot(req2).await.unwrap();

    // Filter by project_id
    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/api-tokens?project_id=proj1")
        .header("authorization", format!("Bearer {}", jwt))
        .body(Body::empty())
        .unwrap();
    
    let res = router.oneshot(req).await.unwrap();
    let (_, body): (StatusCode, PaginatedTokensResponse) = extract_json_response(res).await;
    assert_eq!(body.data.len(), 1);
    assert_eq!(body.data[0].project_id, Some("proj1".to_string()));
}
