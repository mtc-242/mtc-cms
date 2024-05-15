use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use axum::async_trait;

use crate::CFG;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::Result;
use crate::error::session_error::SessionError;
use crate::model::StringListModel;
use crate::model::user_model::{UserCreateModel, UserModel, UserUpdateModel};
use crate::paginator::RepositoryPaginate;
use crate::provider::database_provider::DB;
use crate::repository_paginate;

pub struct UserRepository;

repository_paginate!(UserRepository, UserModel, "users");

#[async_trait]
pub trait UserRepositoryTrait {
    async fn find(&self, id: &str) -> Result<UserModel>;
    async fn find_by_login(&self, login: &str) -> Result<UserModel>;
    async fn create(&self, model: UserCreateModel) -> Result<UserModel>;
    async fn update(&self, id: &str, model: UserUpdateModel) -> Result<UserModel>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn permissions(&self, id: &str) -> Result<Vec<String>>;
    async fn roles(&self, id: &str) -> Result<Vec<String>>;
    async fn groups(&self, id: &str) -> Result<Vec<String>>;
    async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
    async fn unassign_role(&self, user_id: &str, role_id: &str) -> Result<()>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn find(
        &self,
        id: &str,
    ) -> Result<UserModel> {
        let result: Option<UserModel> = DB.query(r#"
            SELECT * FROM type::thing($table, $id);
            "#)
            .bind(("table", "users"))
            .bind(("id", id.to_string()))
            .await?
            .take(0)?;

        match result {
            Some(value) => Ok(value),
            _ => Err(ApiError::from(DbError::EntryNotFound))
        }
    }

    async fn find_by_login(
        &self,
        login: &str,
    ) -> Result<UserModel> {
        let result: Option<UserModel> = DB.query(r#"
            SELECT * FROM type::table($table) WHERE login=$login;
            "#)
            .bind(("table", "users"))
            .bind(("login", login.to_string()))
            .await?
            .take(0)?;

        match result {
            Some(value) => Ok(value),
            _ => Err(ApiError::from(DbError::EntryNotFound))
        }
    }

    async fn create(
        &self,
        model: UserCreateModel,
    ) -> Result<UserModel> {
        let password = model.password.as_bytes();
        let salt = match SaltString::from_b64(&CFG.password_salt) {
            Ok(value) => value,
            _ => Err(ApiError::from(SessionError::PasswordHash))?
        };

        let argon2 = Argon2::default();
        let password_hash = match argon2
            .hash_password(password, &salt) {
            Ok(value) => value.to_string(),
            _ => Err(ApiError::from(SessionError::PasswordHash))?
        };

        let result: Option<UserModel> = DB.query(r#"
            CREATE type::table($table) CONTENT {
	            login: $login,
	            password: $password
            };
            "#)
            .bind(("table", "users"))
            .bind(("name", model.login))
            .bind(("password", password_hash))
            .await?
            .take(0)?;

        match result {
            Some(value) => Ok(value),
            _ => Err(ApiError::from(DbError::EntryAlreadyExists))
        }
    }

    async fn update(
        &self,
        id: &str,
        model: UserUpdateModel) -> Result<UserModel> {
        let result: Option<UserModel> = DB.query(r#"
            UPDATE type::thing($table, $id) MERGE {
	            login: $login
            } WHERE id;
            "#)
            .bind(("table", "users"))
            .bind(("id", id))
            .bind(("login", model.login))
            .await?.take(0)?;

        match result {
            Some(value) => Ok(value),
            _ => Err(ApiError::from(DbError::EntryUpdate))
        }
    }

    async fn delete(
        &self,
        id: &str,
    ) -> Result<()> {
        match DB.query(r#"
            BEGIN TRANSACTION;
            DELETE type::thing($table, $id);
            DELETE FROM type::table($rel_table) WHERE IN = type::thing($table, $id) OR OUT = type::thing($table, $id);
            COMMIT TRANSACTION;
            "#)
            .bind(("table", "users"))
            .bind(("id", id))
            .bind(("rel_table", "user_roles"))
            .await {
            Ok(..) => Ok(()),
            Err(_) => Err(ApiError::from(DbError::EntryDelete))
        }
    }

    async fn permissions(
        &self,
        id: &str,
    ) -> Result<Vec<String>> {
        let result: Option<StringListModel> = DB.query(r#"
            SELECT array::distinct(->user_roles->roles->role_permissions->permissions.name) as items
            FROM type::thing($table, $id);
            "#)
            .bind(("table", "users"))
            .bind(("id", id.to_string()))
            .await?
            .take(0)?;

        match result {
            Some(value) => Ok(value.items),
            _ => Err(ApiError::from(DbError::EntryNotFound))
        }
    }

    async fn roles(
        &self,
        id: &str,
    ) -> Result<Vec<String>> {
        let result: Option<StringListModel> = DB.query(r#"
            SELECT array::distinct(->user_roles->roles.name) as items
            FROM type::thing($table, $id);
            "#)
            .bind(("table", "users"))
            .bind(("id", id.to_string()))
            .await?
            .take(0)?;

        match result {
            Some(value) => Ok(value.items),
            _ => Err(ApiError::from(DbError::EntryNotFound))
        }
    }

    async fn groups(
        &self,
        id: &str,
    ) -> Result<Vec<String>> {
        let result: Option<StringListModel> = DB.query(r#"
            SELECT array::distinct(->user_groups->groups.name) as items
            FROM type::thing($table, $id);
            "#)
            .bind(("table", "users"))
            .bind(("id", id.to_string()))
            .await?
            .take(0)?;

        match result {
            Some(value) => Ok(value.items),
            _ => Err(ApiError::from(DbError::EntryNotFound))
        }
    }

    async fn assign_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        match DB.query(format!(r#"
            RELATE users:{}->user_roles->roles:{};
            "#, user_id, role_id))
            .await {
            Ok(..) => Ok(()),
            Err(_) => Err(ApiError::from(DbError::EntryUpdate))
        }
    }

    async fn unassign_role(
        &self,
        user_id: &str,
        role_id: &str,
    ) -> Result<()> {
        match DB.query(r#"
            DELETE type::thing('users', $user)->user_roles WHERE out=type::thing('roles', $role);
            "#)
            .bind(("user", user_id))
            .bind(("role", role_id))
            .await {
            Ok(..) => Ok(()),
            Err(_) => Err(ApiError::from(DbError::EntryDelete))
        }
    }
}