use axum::async_trait;

use mtc_model::group_model::*;
use mtc_model::list_model::{RecordListModel, StringListModel};
use mtc_model::record_model::RecordModel;

use crate::error::db_error::DbError;
use crate::error::Result;
use crate::repository::RepositoryPaginate;
use crate::repository_paginate;
use crate::service::group_service::GroupService;

repository_paginate!(GroupService, GroupModel, "groups");

#[async_trait]
pub trait GroupRepositoryTrait {
    async fn all(&self) -> Result<RecordListModel>;
    async fn find_by_slug(&self, slug: &str) -> Result<GroupModel>;
    async fn find_by_user(&self, login: &str) -> Result<StringListModel>;
    async fn create(&self, auth: &str, slug: &str, model: GroupCreateModel) -> Result<GroupModel>;
    async fn update(&self, auth: &str, slug: &str, model: GroupUpdateModel) -> Result<GroupModel>;
    async fn delete(&self, slug: &str) -> Result<()>;
}

#[async_trait]
impl GroupRepositoryTrait for GroupService {
    async fn all(&self) -> Result<RecordListModel> {
        Ok(RecordListModel {
            list: self
                .db
                .query(
                    r#"
                    SELECT slug, title from groups;
                    "#,
                )
                .await?
                .take::<Vec<RecordModel>>(0)?,
        })
    }

    async fn find_by_slug(&self, slug: &str) -> Result<GroupModel> {
        self.db
            .query(
                r#"
                SELECT * FROM groups WHERE slug=$slug;
                "#,
            )
            .bind(("slug", slug))
            .await?
            .take::<Option<GroupModel>>(0)?
            .ok_or(DbError::EntryNotFound.into())
    }

    async fn find_by_user(&self, login: &str) -> Result<StringListModel> {
        self.db.query(r#"
            SELECT array::sort(array::distinct(->user_groups->groups.slug)) as list FROM users WHERE login=$login
            "#)
            .bind(("login", login))
            .await?
            .take::<Option<StringListModel>>(0)?
            .ok_or(DbError::EntryNotFound.into())
    }

    async fn create(&self, auth: &str, slug: &str, model: GroupCreateModel) -> Result<GroupModel> {
        self.db
            .query(
                r#"
                CREATE groups CONTENT {
	                slug: $slug,
	                title: $title,
	                created_by: $auth_id,
	                updated_by: $auth_id
                };
            "#,
            )
            .bind(("auth_id", auth))
            .bind(("slug", slug))
            .bind(("title", model.title))
            .await?
            .take::<Option<GroupModel>>(0)?
            .ok_or(DbError::EntryNotFound.into())
    }

    async fn update(&self, auth: &str, slug: &str, model: GroupUpdateModel) -> Result<GroupModel> {
        self.db
            .query(
                r#"
                UPDATE groups MERGE {
	                title: $title,
	                updated_by: $auth_id
                } WHERE slug=$slug;
                "#,
            )
            .bind(("auth_id", auth))
            .bind(("slug", slug))
            .bind(("title", model.title))
            .await?
            .take::<Option<GroupModel>>(0)?
            .ok_or(DbError::EntryUpdate.into())
    }

    async fn delete(&self, slug: &str) -> Result<()> {
        self.db
            .query(
                r#"
                DELETE FROM groups WHERE slug=$slug;
                "#,
            )
            .bind(("slug", slug))
            .await?;

        Ok(())
    }
}
