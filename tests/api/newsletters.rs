use reqwest::Response;
use serde_json::Value;
use wiremock::http::Method::Post;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

use zero2prod::startup::Application;

use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange

    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!( {
"title" : "Newsletter title" ,
"content" : {
"text" : "Newsletter body as plain text" ,
"html" : "<p>Newsletter body as HTML</p>" ,
}
} );
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_request_body = serde_json::json!( {
"title" : "Newsletter title" ,
"content" : {
"text" : "Newsletter body as plain text" ,
"html" : "<p>Newsletter body as HTML</p>" ,
}
} );
    let response = app.post_newsletter(&newsletter_request_body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}
async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let _mock_guard = Mock::given(path("/email"))
        .and(method(Post))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}
async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            serde_json::json!( {
            "content" : {
            "text" : "Newsletter body as plain text" ,
            "html" : "<p>Newsletter body as HTML</p>" ,
            }
            } ),
            "missing title",
        ),
        (
            serde_json::json!( { "title" : "Newsletter!" } ),
            "missing content",
        ),
    ];

    for (invalid_body, error_messages) in test_cases {
        let response = app.post_newsletter( &invalid_body).await;

        // Assert

        assert_eq!(400, response.status().as_u16());
    }
}


