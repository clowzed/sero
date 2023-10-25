use entity::prelude::*;
use sea_orm::prelude::*;
use sea_orm::Set;
use sea_orm::TransactionTrait;

pub struct UsersService;

pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

impl std::fmt::Debug for UserCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserCredentials")
            .field("username", &self.username)
            .finish()
    }
}

impl UserCredentials {
    #[inline]
    pub fn password_hash(&self) -> String {
        sha256::digest(&self.password)
    }
}

impl UsersService {
    #[tracing::instrument(skip(connection))]
    pub async fn find<T: ConnectionTrait + TransactionTrait>(
        credentials: UserCredentials,
        connection: &T,
    ) -> Result<Option<User>, DbErr> {
        UserEntity::find()
            .filter(
                UserColumn::Username
                    .eq(&credentials.username)
                    .and(UserColumn::Password.eq(&credentials.password_hash())),
            )
            .one(connection)
            .await
    }

    #[tracing::instrument(skip(connection))]
    pub async fn find_by_id<T: ConnectionTrait + TransactionTrait>(
        id: i32,
        connection: &T,
    ) -> Result<Option<User>, DbErr> {
        UserEntity::find_by_id(id).one(connection).await
    }

    #[tracing::instrument(skip(connection))]
    pub async fn find_by_username<T: ConnectionTrait + TransactionTrait>(
        username: &str,
        connection: &T,
    ) -> Result<Option<User>, DbErr> {
        UserEntity::find()
            .filter(UserColumn::Username.eq(username))
            .one(connection)
            .await
    }

    #[tracing::instrument(skip(connection))]
    pub async fn new_user<T: ConnectionTrait + TransactionTrait>(
        credentials: UserCredentials,
        connection: &T,
    ) -> Result<User, DbErr> {
        UserEntity::insert(ActiveUser {
            username: Set(credentials.username.clone()),
            password: Set(credentials.password_hash().clone()),
            ..Default::default()
        })
        .exec_with_returning(connection)
        .await
    }

    #[tracing::instrument(skip(connection))]
    pub async fn count<T: ConnectionTrait + TransactionTrait>(
        connection: &T,
    ) -> Result<u64, DbErr> {
        UserEntity::find().count(connection).await
    }
}
