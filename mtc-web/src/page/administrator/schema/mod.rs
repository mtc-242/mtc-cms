use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_std::i18n::use_i18;
use dioxus_std::translate;

use mtc_model::pagination_model::PaginationModel;

use crate::component::loading_box::LoadingBoxComponent;
use crate::component::paginator::{PaginatorComponent, PaginatorComponentMode};
use crate::component::reloading_box::ReloadingBoxComponent;
use crate::handler::schema_handler::SchemaHandler;
use crate::model::page_action::PageAction;
use crate::page::administrator::schema::editor::SchemaEditor;
use crate::APP_STATE;

mod editor;

#[component]
pub fn Schema() -> Element {
    let app_state = APP_STATE.peek();
    let auth_state = app_state.auth.read_unchecked();
    let i18 = use_i18();

    let page = use_signal(|| 1usize);
    let mut page_action = use_context_provider(|| Signal::new(PageAction::None));

    let pagination = use_signal(|| PaginationModel::new(0, 10));
    let mut schemas_future =
        use_resource(move || async move { APP_STATE.peek().api.get_schema_list(page()).await });

    use_effect(move || {
        if page_action() == PageAction::None {
            schemas_future.restart()
        }
    });

    if page_action().ne(&PageAction::None) {
        return rsx! { SchemaEditor {} };
    }

    rsx! {
        match &*schemas_future.read() {
            Some(Ok(response)) => rsx! {
                section { class: "flex grow flex-col items-center gap-3 p-2 body-scroll",
                    table { class: "table w-full",
                        thead {
                            tr {
                                th { class: "w-6" }
                                th { { translate!(i18, "messages.slug") } }
                                th { { translate!(i18, "messages.title") } }
                            }
                        }
                        tbody {
                            for item in response.data.iter(){
                                {
                                    let m_slug = item.slug.clone();
                                    rsx! {
                                        tr { class: "cursor-pointer hover:bg-base-200 hover:shadow-md",
                                            onclick: move |_| page_action.set(PageAction::Item(m_slug.clone())),
                                            td { class: "text-primary",
                                                if item.is_system {
                                                    Icon {
                                                        width: 16,
                                                        height: 16,
                                                        fill: "currentColor",
                                                        icon: dioxus_free_icons::icons::md_action_icons::MdLock
                                                    }
                                                } else if item.is_collection {
                                                    Icon {
                                                        width: 16,
                                                        height: 16,
                                                        fill: "currentColor",
                                                        icon: dioxus_free_icons::icons::io_icons::IoBook
                                                    }
                                                } else {
                                                    Icon {
                                                        width: 16,
                                                        height: 16,
                                                        fill: "currentColor",
                                                        icon: dioxus_free_icons::icons::fa_regular_icons::FaFile
                                                    }
                                                }
                                            }
                                            td { { item.slug.clone() } }
                                            td { { item.title.clone() } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    PaginatorComponent { mode: PaginatorComponentMode::Full, page, pagination: response.pagination.clone().unwrap_or_default() }
                }
                button {
                    class: "absolute right-4 bottom-4 btn btn-circle btn-neutral",
                    onclick: move |_| page_action.set(PageAction::New),
                    Icon {
                        width: 26,
                        height: 26,
                        icon: dioxus_free_icons::icons::md_content_icons::MdAdd
                    }
                }
            },
            Some(Err(e)) => rsx! { ReloadingBoxComponent { message: e.message(), resource: schemas_future } },
            None => rsx! { LoadingBoxComponent {} },
        }
    }
}
/*
    let app_state = APP_STATE.peek();
    let auth_state = app_state.auth.read_unchecked();

    if !auth_state.is_permission("schema::read") {
        return rsx! { NotFoundPage {} };
    }

    let page = use_signal(|| 1usize);
    let schema_selected = use_context_provider(|| Signal::new(PageAction::None));

    let mut schemas = use_context_provider(|| Signal::new(BTreeMap::<usize, SchemaModel>::new()));
    let mut pagination = use_context_provider(|| Signal::new(PaginationModel::new(0, 10)));
    let mut schema_future =
        use_resource(move || async move { APP_STATE.peek().api.get_schema_list(page()).await });

    use_effect(move || {
        if schema_selected() == PageAction::None {
            schema_future.restart()
        }
    });

    if schema_selected() == PageAction::None {
        match &*schema_future.read_unchecked() {
            Some(Ok(response)) => {
                let mut schema_list = BTreeMap::<usize, SchemaModel>::new();

                for (count, item) in response.data.iter().enumerate() {
                    schema_list.insert(count, item.clone());
                }

                schemas.set(schema_list);
                pagination.set(
                    response
                        .pagination
                        .clone()
                        .unwrap_or(PaginationModel::new(0, 10)),
                );

                rsx! { SchemaList { page } }
            }
            Some(Err(e)) => {
                rsx! { ReloadingBoxComponent { message: e.message(), resource: schema_future } }
            }
            None => rsx! { LoadingBoxComponent {} },
        }
    } else {
        rsx! { SchemaSingle {} }
    }
}


 */
