#[cfg(test)]
pub mod tests {
    use crate::{
        api::auth::registration::{request::RegistrationRequest, tests::call::tests::registration},
        app,
    };
    use axum::http::StatusCode;
    use axum_test::TestServer as TestClient;
    use entity::prelude::*;
    use sea_orm::EntityTrait;
    use uuid::Uuid;

    #[tokio::test]
    async fn occupied() {
        dotenvy::from_filename_override(".env.tests").ok();

        //? Set maximum users allowed for registration
        std::env::set_var("MAX_USERS", "2");

        let (app, state) = app().await.expect("Failed to initialize application!");
        //? Perform drop for users
        UserEntity::delete_many()
            .exec(state.connection())
            .await
            .expect("Failed to delete all users");

        let client = TestClient::new(app).expect("Failed to run server for testing");

        let first_user_registration_request = RegistrationRequest {
            login: Uuid::new_v4().into(),
            password: Uuid::new_v4().into(),
        };

        let first_user_registration_response = registration(&client, &first_user_registration_request).await;
        assert!(first_user_registration_response.is_ok());

        let first_user_second_attempt_registration_response =
            registration(&client, &first_user_registration_request).await;
        assert!(first_user_second_attempt_registration_response.is_err_and(|error| error.0 == StatusCode::CONFLICT));
    }
}
