use crate::api::{start_session, StartSessionRequest};
use crate::components::all_sessions::SessionEditor;
use crate::components::timer::Timer;
use crate::models::Session;
use leptos::*;

#[component]
pub fn ActiveSessions(
    token: String,
    sessions: Resource<String, Result<Vec<Session>, String>>,
) -> impl IntoView {
    let token_stored = store_value(token);
    let (expanded_id, set_expanded_id) = create_signal(None::<uuid::Uuid>);

    let toggle_expand = move |id: uuid::Uuid| {
        set_expanded_id.update(|current| {
            if Some(id) == *current {
                *current = None;
            } else {
                *current = Some(id);
            }
        });
    };

    view! {
        <div>
            <h2>"Aktive Sitzungen"</h2>
            <Transition fallback=move || view! { <p class="status-msg">"Lade..."</p> }>
                {move || {
                    sessions.get().map(|res| match res {
                        Ok(data) => {
                            if data.is_empty() {
                                view! { <p class="status-msg">"Keine aktiven Sitzungen vorhanden."</p> }.into_view()
                            } else {
                                view! {
                                    <div class="sessions-list">
                                        {data.into_iter().map(|s| {
                                            let s_id = s.id;
                                            let s_stored = store_value(s);
                                            let is_expanded = move || expanded_id.get() == Some(s_id);

                                            let on_quick_clone = {
                                                let t = token_stored.get_value();
                                                move |ev: leptos::ev::MouseEvent| {
                                                    ev.stop_propagation();
                                                    let s = s_stored.get_value();
                                                    let payload = StartSessionRequest {
                                                        description: s.description.clone(),
                                                        duration_minutes: s.duration.secs / 60,
                                                        tags: Some(s.tags.clone()),
                                                        notes: Some(s.notes.clone()),
                                                        ratings: s.ratings.clone(),
                                                    };
                                                    let t = t.clone();
                                                    spawn_local(async move {
                                                        if let Ok(_) = start_session(t, payload).await {
                                                            sessions.refetch();
                                                        }
                                                    });
                                                }
                                            };

                                            view! {
                                                <div class="session-item-container active-session-item" class:expanded=is_expanded>
                                                    <div
                                                        class="session-summary"
                                                        on:click=move |_| toggle_expand(s_id)
                                                    >
                                                        <div class="session-main-info">
                                                            <span class="session-desc">{move || s_stored.get_value().description}</span>
                                                            <Timer session=s_stored.get_value() />
                                                        </div>
                                                        <div class="session-meta-info">
                                                            <button
                                                                class="quick-clone-btn"
                                                                title="Klonen & Starten"
                                                                on:click=on_quick_clone
                                                            >
                                                                "▶"
                                                            </button>
                                                            <span class=move || format!("state-tag {}", s_stored.get_value().state.to_lowercase())>
                                                                {move || s_stored.get_value().state}
                                                            </span>
                                                            <span class="expand-icon">{move || if is_expanded() { "▲" } else { "▼" }}</span>
                                                        </div>
                                                    </div>

                                                    <Show when=is_expanded>
                                                        <SessionEditor
                                                            token=token_stored.get_value()
                                                            session=s_stored.get_value()
                                                            on_updated=move |_| {
                                                                sessions.refetch();
                                                            }
                                                            on_delete=move |_| {
                                                                sessions.refetch();
                                                            }
                                                        />
                                                    </Show>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        },
                        Err(e) => view! { <p class="status-msg" style="color: var(--accent-color)">{e}</p> }.into_view(),
                    })
                }}
            </Transition>
        </div>
    }
}
