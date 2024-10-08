use chrono::Local;
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_std::i18n::use_i18;
use dioxus_std::translate;
use serde_json::{Map, Value};

use html_field::HtmlField;
use mtc_model::api_model::{ApiModel, ApiPostModel};
use mtc_model::auth_model::AuthModelTrait;
use mtc_model::field_model::FieldTypeModel;
use mtc_model::record_model::RecordModel;
use mtc_model::schema_model::SchemaModel;
use string_field::StringField;
use text_field::TextField;

use crate::APP_STATE;
use crate::component::loading_box::LoadingBoxComponent;
use crate::handler::content_handler::ContentHandler;
use crate::handler::schema_handler::SchemaHandler;
use crate::model::modal_model::ModalModel;
use crate::page::administrator::storage::StorageManager;
use crate::page::not_found::NotFoundPage;
use crate::repository::storage::use_session_storage;
use crate::service::content_service::ContentService;
use crate::service::validator_service::ValidatorService;

mod html_field;
mod string_field;
mod text_field;

#[derive(Props, Clone, PartialEq)]
pub struct FieldProps {
    pub slug: String,
    pub title: String,
    pub value: Value,
}

#[component]
pub fn EditorPage(schema_prop: String, content_prop: String) -> Element {
    let app_state = APP_STATE.peek();
    let auth_state = app_state.auth.read();
    let i18 = use_i18();

    if !auth_state.is_permission("writer") | !auth_state.is_permission("schema::read") {
        return rsx! { NotFoundPage {} };
    }

    let mut is_busy = use_signal(|| true);

    let mut schema = use_signal(SchemaModel::default);
    let mut schema_slug = use_signal(|| schema_prop.clone());
    let mut content = use_signal(ApiModel::default);
    let mut content_slug = use_signal(|| content_prop.clone());
    let storage = use_memo(move || content().id);
    let mut form_published = use_signal(|| false);

    let mut is_public_storage_shown = use_signal(|| false);
    let mut is_private_storage_shown = use_signal(|| false);

    let mut content_id = use_session_storage("contentId", String::new);

    let compare_schema_slug = schema_slug();
    let compare_content_slug = content_slug();
    use_effect(use_reactive(
        (&schema_prop, &content_prop),
        move |(schema_prop, content_prop)| {
            if compare_schema_slug.ne(&schema_prop) {
                schema_slug.set(schema_prop)
            }
            if compare_content_slug.ne(&content_prop) {
                content_slug.set(content_prop)
            }
        }));

    let mut breadcrumbs = app_state.breadcrumbs.signal();
    use_effect(move || {
        breadcrumbs.set(
            if schema_slug().eq("singles") {
                vec![
                    RecordModel { title: translate!(i18, "messages.content"), slug: "".to_string() },
                    RecordModel { title: translate!(i18, "messages.singles"), slug: format!("/content/{}", schema_slug()) },
                    RecordModel { title: content().title, slug: "".to_string() },
                ]
            } else {
                vec![
                    RecordModel { title: translate!(i18, "messages.content"), slug: "".to_string() },
                    RecordModel { title: translate!(i18, "messages.collections"), slug: "".to_string() },
                    RecordModel { title: schema().title.clone(), slug: format!("/content/{}", schema_slug()) },
                    RecordModel { title: content().title, slug: "".to_string() },
                ]
            }
        );
    });

    use_effect(move || {
        let app_state = APP_STATE.peek();

        let m_schema_slug = schema_slug();
        let m_content_slug = content_slug();

        spawn(async move {
            if m_schema_slug.eq("singles") {
                match APP_STATE.peek().api.get_schema(&m_content_slug).await {
                    Ok(value) => schema.set(value),
                    Err(e) => {
                        app_state.modal.signal().set(ModalModel::Error(e.message()));
                        navigator().go_back()
                    }
                }
            } else {
                match APP_STATE.peek().api.get_schema(&m_schema_slug).await {
                    Ok(value) => schema.set(value),
                    Err(e) => {
                        app_state.modal.signal().set(ModalModel::Error(e.message()));
                        navigator().go_back()
                    }
                }
            }
            if schema().is_collection {
                match app_state
                    .api
                    .get_collection_content(&schema().slug, &m_content_slug)
                    .await
                {
                    Ok(value) => {
                        form_published.set(value.published);
                        content_id.set(value.id.clone());
                        content.set(value)
                    }
                    Err(e) => {
                        app_state.modal.signal().set(ModalModel::Error(e.message()));
                        navigator().go_back()
                    }
                }
            } else {
                match app_state.api.get_single_content(&schema().slug).await {
                    Ok(value) => {
                        form_published.set(value.published);
                        content_id.set(value.id.clone());
                        content.set(value)
                    }
                    Err(e) => {
                        app_state.modal.signal().set(ModalModel::Error(e.message()));
                        navigator().go_back()
                    }
                }
            }

            is_busy.set(false);
        });
    });

    let schema_permission = use_memo(move || {
        if schema().is_public {
            "content".to_string()
        } else {
            schema().slug.clone()
        }
    });

    let submit_task = move |event: Event<FormData>| {
        if !event.is_title_valid() {
            APP_STATE
                .peek()
                .modal
                .signal()
                .set(ModalModel::Error(translate!(i18, "errors.fields")));
            return;
        }
        is_busy.set(true);

        let mut submit_fields = Map::new();
        if let Some(fields) = schema().fields {
            fields.iter().for_each(|field| {
                submit_fields.insert(
                    field.slug.clone(),
                    Value::String(event.get_string(&field.slug)),
                );
            });
        }

        let submit_form = ApiPostModel {
            title: event.get_string("title"),
            published: event.get_string_option("published").is_some(),
            fields: match submit_fields.is_empty() {
                true => None,
                false => Some(Value::Object(submit_fields.clone())),
            },
        };

        let t_schema = schema().slug.clone();
        let t_content = content().slug.clone();

        spawn(async move {
            match APP_STATE
                .peek()
                .api
                .update_content(
                    match &schema().is_collection {
                        true => &t_schema,
                        false => "singles",
                    },
                    &t_content,
                    &submit_form,
                )
                .await
            {
                Ok(_) => navigator().go_back(),
                Err(e) => {
                    let content_model = ApiModel {
                        id: content().id.clone(),
                        slug: content().slug.clone(),
                        title: event.get_string("title"),
                        fields: match submit_fields.is_empty() {
                            true => None,
                            false => Some(Value::Object(submit_fields)),
                        },
                        published: event.get_string_option("published").is_some(),
                        created_at: content().created_at.clone(),
                        updated_at: content().updated_at.clone(),
                        created_by: content().created_by.clone(),
                        updated_by: content().updated_by.clone(),
                    };

                    content.set(content_model);

                    APP_STATE
                        .peek()
                        .modal
                        .signal()
                        .set(ModalModel::Error(e.message()))
                }
            }
            is_busy.set(false);
        });
    };

    let content_delete = move |_| {
        spawn(async move {
            match APP_STATE
                .peek()
                .api
                .delete_content(&schema().slug, &content().slug)
                .await
            {
                Ok(_) => navigator().go_back(),
                Err(e) => APP_STATE
                    .peek()
                    .modal
                    .signal()
                    .set(ModalModel::Error(e.message())),
            }
            is_busy.set(false);
        });
    };

    if is_busy() {
        return rsx! {
            div { class: crate::DIV_CENTER,
                LoadingBoxComponent {}
            }
        };
    }

    rsx! {
        if is_public_storage_shown() {
            StorageManager { dir: storage, is_shown: is_public_storage_shown, private: false }
        } else if is_private_storage_shown() {
            StorageManager { dir: storage, is_shown: is_private_storage_shown, private: true }
        }
        section { class: "flex grow select-none flex-row gap-6",
            form { class: "flex grow flex-col items-center gap-3",
                id: "content-form",
                autocomplete: "off",
                onsubmit: submit_task,

                label { class: "w-full form-control",
                    div { class: "label",
                        span { class: "label-text text-primary", { translate!(i18, "messages.slug") } }
                    }
                    input { r#type: "text", name: "slug",
                        class: "input input-bordered",
                        disabled: true,
                        minlength: 4,
                        maxlength: 30,
                        required: true,
                        pattern: crate::SLUG_PATTERN,
                        initial_value: content().slug.clone()
                    }
                    span {}
                }
                label { class: "w-full form-control",
                    div { class: "label",
                        span { class: "label-text text-primary", { translate!(i18, "messages.title") } }
                    }
                    input { r#type: "text", name: "title",
                        class: "input input-bordered",
                        minlength: 4,
                        maxlength: 50,
                        required: true,
                        pattern: crate::TITLE_PATTERN,
                        initial_value: content().title.clone()
                    }
                    span {}
                }

                for field in schema().fields.unwrap_or(vec![]).iter() {
                    match field.field_type {
                        FieldTypeModel::Html => rsx! {
                            HtmlField { slug: field.slug.clone(), title: field.title.clone(), value: content.extract_field(&field.slug) }
                        },
                        FieldTypeModel::Text => rsx! {
                            TextField { slug: field.slug.clone(), title: field.title.clone(), value: content.extract_field(&field.slug) }
                        },
                        _ => rsx! {
                            StringField { slug: field.slug.clone(), title: field.title.clone(), value: content.extract_field(&field.slug) }
                        }
                    }
                }
            }

            aside { class: "flex flex-col gap-3 pt-5 min-w-36",
                button { class: "btn btn-ghost",
                    onclick: move |_| navigator().go_back(),
                    Icon {
                        width: 22,
                        height: 22,
                        icon: dioxus_free_icons::icons::md_navigation_icons::MdArrowBack
                    }
                    { translate!(i18, "messages.cancel") }
                }
                div { class: "flex flex-col gap-1 rounded border p-2 input-bordered label-text",
                    span { class: "italic label-text text-primary", { translate!(i18, "messages.created_at") } ":" }
                    span { { content.read().created_by.clone() } }
                    span { class: "label-text-alt", { content().created_at.clone().with_timezone(&Local).format("%H:%M %d/%m/%Y").to_string() } }
                    span { class: "mt-1 italic label-text text-primary", { translate!(i18, "messages.updated_at") } ":" }
                    span { { content.read().updated_by.clone() } }
                    span { class: "label-text-alt", { content().updated_at.clone().with_timezone(&Local).format("%H:%M %d/%m/%Y").to_string() } }
                }
                if auth_state.is_permission(&[&schema_permission(), "::write"].concat()) {
                    label { class:
                        if form_published() {
                            "items-center p-3 swap text-success"
                        } else {
                            "items-center p-3 swap text-warning"
                        },
                        input { r#type: "checkbox",
                            name: "published",
                            form: "content-form",
                            checked: form_published(),
                            onchange: move |event| form_published.set(event.checked())
                        }
                        div { class: "inline-flex gap-3 swap-on",
                            Icon {
                                width: 22,
                                height: 22,
                                fill: "currentColor",
                                icon: dioxus_free_icons::icons::md_action_icons::MdVisibility
                            }
                            { translate!(i18, "messages.published") }
                        }
                        div { class: "inline-flex gap-3 swap-off",
                            Icon {
                                width: 22,
                                height: 22,
                                fill: "currentColor",
                                icon: dioxus_free_icons::icons::md_action_icons::MdVisibilityOff
                            }
                            { translate!(i18, "messages.draft") }
                        }
                    }

                    div { class: "w-full join",
                        if auth_state.is_permission("storage::read") {
                            button { class: "btn btn-ghost join-item",
                                onclick: move |_| is_public_storage_shown.set(true),
                                Icon {
                                    width: 22,
                                    height: 22,
                                    fill: "currentColor",
                                    icon: dioxus_free_icons::icons::md_social_icons::MdGroups
                                }
                            }
                        } else {
                            button { class: "btn btn-ghost btn-disabled join-item",
                                disabled: "disabled",
                                Icon {
                                    width: 22,
                                    height: 22,
                                    fill: "currentColor",
                                    icon: dioxus_free_icons::icons::md_social_icons::MdGroups
                                }
                            }
                        }
                        div { class: "grid place-items-center w-fit join-item text-neutral px-2 text-lg semibold",
                            "<>"
                        }
                    /*
                    div { class: "grid place-items-center w-full join-item bg-base-content text-base-300",
                        Icon {
                            width: 30,
                            height: 30,
                            fill: "currentColor",
                            icon: dioxus_free_icons::icons::md_file_icons::MdCloudUpload
                        }
                    }
                     */
                        if auth_state.is_permission("private_storage::read") {
                            button { class: "btn btn-ghost join-item",
                                onclick: move |_| is_private_storage_shown.set(true),
                                Icon {
                                    width: 22,
                                    height: 22,
                                    fill: "currentColor",
                                    icon: dioxus_free_icons::icons::md_content_icons::MdShield
                                }
                            }
                        } else {
                            button { class: "btn btn-ghost btn-disabled join-item",
                                disabled: "disabled",
                                Icon {
                                    width: 22,
                                    height: 22,
                                    fill: "currentColor",
                                    icon: dioxus_free_icons::icons::md_content_icons::MdShield
                                }
                            }
                        }
                    }
                
                    button { class: "btn btn-primary",
                        r#type: "submit",
                        form: "content-form",
                        Icon {
                            width: 22,
                            height: 22,
                            fill: "currentColor",
                            icon: dioxus_free_icons::icons::md_content_icons::MdSave
                        }
                        { translate!(i18, "messages.save") }
                    }
                }
                if schema().is_collection && auth_state.is_permission(&[&schema_permission(), "::delete"].concat()) {
                    div { class: "divider" }
                    button { class: "btn btn-ghost text-error",
                        onclick: content_delete,
                        Icon {
                            width: 18,
                            height: 18,
                            fill: "currentColor",
                            icon: dioxus_free_icons::icons::fa_regular_icons::FaTrashCan
                        }
                        { translate!(i18, "messages.delete") }
                    }
                }
            }
        }
    }
}
