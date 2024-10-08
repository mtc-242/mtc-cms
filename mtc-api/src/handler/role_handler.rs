use std::sync::Arc;

use axum::extract::{Path, State};
use tower_sessions::Session;
use tracing::{error, warn};

use mtc_model::list_model::{RecordListModel, StringListModel};
use mtc_model::pagination_model::{PaginationBuilder, PaginationModel};
use mtc_model::role_model::{RoleCreateModel, RoleModel, RoleUpdateModel};

use crate::handler::Result;
use crate::middleware::auth_middleware::UserSession;
use crate::model::request_model::ValidatedPayload;
use crate::model::response_model::{ApiResponse, HandlerResult};
use crate::repository::permissions_repository::PermissionsRepositoryTrait;
use crate::repository::role_repository::RoleRepositoryTrait;
use crate::repository::RepositoryPaginate;
use crate::state::AppState;

pub async fn role_all_handler(
    state: State<Arc<AppState>>,
    session: Session,
) -> Result<RecordListModel> {
    session.permission("role::read").await?;

    state.role_service.all().await?.ok_model()
}

pub async fn role_list_handler(
    page: Option<Path<usize>>,
    state: State<Arc<AppState>>,
    session: Session,
) -> Result<Vec<RoleModel>> {
    session.permission("role::read").await?;

    let page: usize = match page {
        Some(Path(value)) => value,
        _ => 1,
    };

    let pagination = PaginationModel::new(
        state.role_service.get_total().await?,
        state.cfg.rows_per_page,
    )
    .page(page);

    state
        .role_service
        .get_page(pagination.from, pagination.per_page)
        .await?
        .ok_page(pagination)
}

pub async fn role_get_handler(
    Path(slug): Path<String>,
    session: Session,
    state: State<Arc<AppState>>,
) -> Result<RoleModel> {
    session.permission("role::read").await?;

    let mut role_model = state.role_service.find_by_slug(&slug).await?;

    let role_permissions = state.permissions_service.find_by_role(&slug).await?;

    if !role_permissions.list.is_empty() {
        role_model.permissions = Some(role_permissions.list);
    }

    role_model.ok_model()
}

pub async fn role_create_handler(
    Path(slug): Path<String>,
    state: State<Arc<AppState>>,
    session: Session,
    ValidatedPayload(payload): ValidatedPayload<RoleCreateModel>,
) -> Result<RoleModel> {
    session.permission("role::write").await?;

    let role_model = state
        .role_service
        .create(&session.auth_id().await?, &slug, &payload)
        .await?;

    if let Some(permissions) = payload.permissions {
        for permission in permissions {
            match state.permissions_service.find_by_slug(&permission).await {
                Ok(value) => {
                    state
                        .role_service
                        .permission_assign(&role_model.id, &value.id)
                        .await?
                }
                _ => warn!("can't find permission -> {permission}"),
            }
        }
    }

    role_model.ok_model()
}

pub async fn role_update_handler(
    Path(slug): Path<String>,
    state: State<Arc<AppState>>,
    session: Session,
    ValidatedPayload(payload): ValidatedPayload<RoleUpdateModel>,
) -> Result<RoleModel> {
    session.permission("role::write").await?;

    let role_model = state
        .role_service
        .update(&session.auth_id().await?, &slug, &payload)
        .await?;

    state.role_service.permissions_drop(&role_model.id).await?;

    if let Some(permissions) = payload.permissions {
        for permission in permissions {
            match state.permissions_service.find_by_slug(&permission).await {
                Ok(value) => {
                    state
                        .role_service
                        .permission_assign(&role_model.id, &value.id)
                        .await?
                }
                _ => warn!("can't find permission -> {permission}"),
            }
        }
    }

    role_model.ok_model()
}

pub async fn role_delete_handler(
    Path(slug): Path<String>,
    state: State<Arc<AppState>>,
    session: Session,
) -> Result<()> {
    session.permission("role::delete").await?;

    state.role_service.delete(&slug).await?.ok_ok()
}

pub async fn role_list_delete_handler(
    state: State<Arc<AppState>>,
    session: Session,
    ValidatedPayload(payload): ValidatedPayload<StringListModel>,
) -> Result<()> {
    session.permission("role::delete").await?;

    for item in payload.list {
        match state.role_service.delete(&item).await {
            Ok(_) => (),
            Err(e) => error!("Role delete: {}", e.to_string()),
        }
    }
    Ok(ApiResponse::Ok)
}

pub async fn role_get_permissions(
    Path(slug): Path<String>,
    session: Session,
    state: State<Arc<AppState>>,
) -> Result<StringListModel> {
    session.permission("role::read").await?;

    state
        .permissions_service
        .find_by_role(&slug)
        .await?
        .ok_model()
}

pub async fn role_set_permissions(
    Path(slug): Path<String>,
    session: Session,
    state: State<Arc<AppState>>,
    ValidatedPayload(payload): ValidatedPayload<StringListModel>,
) -> Result<StringListModel> {
    session.permission("role::write").await?;

    let role_model = state.role_service.find_by_slug(&slug).await?;

    state.role_service.permissions_drop(&role_model.id).await?;

    for permission in payload.list {
        match state.permissions_service.find_by_slug(&permission).await {
            Ok(value) => {
                state
                    .role_service
                    .permission_assign(&role_model.id, &value.id)
                    .await?
            }
            _ => warn!("can't find permission -> {permission}"),
        }
    }

    state
        .permissions_service
        .find_by_role(&slug)
        .await?
        .ok_model()
}
