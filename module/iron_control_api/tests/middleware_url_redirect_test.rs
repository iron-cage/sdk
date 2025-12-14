use iron_control_api::middleware::url_redirect::{ convert_old_path_to_new, build_redirect_uri };
use axum::http::Uri;

#[test]
fn test_convert_old_path_to_new()
{
  assert_eq!( convert_old_path_to_new( "/api/tokens" ), Some( "/api/v1/api-tokens".to_string() ) );
  assert_eq!( convert_old_path_to_new( "/api/tokens/abc123" ), Some( "/api/v1/api-tokens/abc123".to_string() ) );
  assert_eq!( convert_old_path_to_new( "/api/tokens/abc123/rotate" ), Some( "/api/v1/api-tokens/abc123/rotate".to_string() ) );
  assert_eq!( convert_old_path_to_new( "/api/users" ), None );
  assert_eq!( convert_old_path_to_new( "/api/v1/api-tokens" ), None );
}

#[test]
fn test_build_redirect_uri()
{
  let uri: Uri = "/api/tokens".parse().unwrap();
  assert_eq!( build_redirect_uri( &uri, "/api/v1/api-tokens" ), "/api/v1/api-tokens" );

  let uri: Uri = "/api/tokens?page=2&per_page=10".parse().unwrap();
  assert_eq!( build_redirect_uri( &uri, "/api/v1/api-tokens" ), "/api/v1/api-tokens?page=2&per_page=10" );
}
