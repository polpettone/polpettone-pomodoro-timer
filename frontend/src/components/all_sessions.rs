use crate::api::{delete_session, fetch_all_sessions, update_session, UpdateSessionRequest};
use crate::models::{Session, SessionRatings};
use leptos::*;

#[component]
pub fn AllSessions(token: String, #[prop(into)] on_update: Callback<()>) -> impl IntoView {
    let token = store_value(token);
    let (is_open, set_is_open) = create_signal(false);
    let (expanded_id, set_expanded_id) = create_signal(None::<uuid::Uuid>);

    let sessions = create_resource(
        move || (is_open.get(), token.get_value()),
        |(open, t)| async move {
            if open {
                fetch_all_sessions(t).await
            } else {
                Ok(vec![])
            }
        },
    );

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
        <div class="all-sessions">
            <button
                class="expand-btn"
                on:click=move |_| set_is_open.update(|v| *v = !*v)
            >
                {move || if is_open.get() { "Historie ausblenden ▲" } else { "Historie anzeigen ▼" }}
            </button>

            <div class="expand-content" class:open=is_open>
                <Transition fallback=move || view! { <p class="status-msg">"Lade Historie..."</p> }>
                    {move || {
                        sessions.get().map(|res| match res {
                            Ok(data) => {
                                if data.is_empty() {
                                    view! { <p class="status-msg">"Keine Sitzungen in der Historie."</p> }.into_view()
                                } else {
                                    view! {
                                        <div class="sessions-list">
                                            {data.into_iter().map(|s| {
                                                let is_expanded = move || expanded_id.get() == Some(s.id);
                                                let s_clone = s.clone();
                                                view! {
                                                    <div class="session-item-container" class:expanded=is_expanded>
                                                        <div
                                                            class="session-summary"
                                                            on:click={let id = s.id; move |_| toggle_expand(id)}
                                                        >
                                                            <div class="session-main-info">
                                                                <span class="session-time">{&s.start}</span>
                                                                <span class="session-desc">{&s.description}</span>
                                                            </div>
                                                            <div class="session-meta-info">
                                                                <span class="session-duration">{s.duration.secs / 60} "m"</span>
                                                                <span class=format!("state-tag {}", s.state.to_lowercase())>{&s.state}</span>
                                                                <span class="expand-icon">{move || if is_expanded() { "▲" } else { "▼" }}</span>
                                                            </div>
                                                        </div>

                                                        <Show when=is_expanded>
                                                            <SessionEditor
                                                                token=token.get_value()
                                                                session=s_clone.clone()
                                                                on_updated=move |_| {
                                                                    sessions.refetch();
                                                                    on_update.call(());
                                                                }
                                                                on_delete=move |_| {
                                                                    sessions.refetch();
                                                                    on_update.call(());
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
        </div>
    }
}

#[component]
fn SessionEditor(
    token: String,
    session: Session,
    #[prop(into)] on_updated: Callback<()>,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    let token = store_value(token);
    let (description, set_description) = create_signal(session.description.clone());
    let (notes, set_notes) = create_signal(session.notes.clone());
    let (tags, set_tags) = create_signal(session.tags.join(", "));

    let (mental, set_mental) = create_signal(
        session
            .ratings
            .as_ref()
            .map(|r| r.mental_energy)
            .unwrap_or(0),
    );
    let (physical, set_physical) = create_signal(
        session
            .ratings
            .as_ref()
            .map(|r| r.physical_energy)
            .unwrap_or(0),
    );
    let (cognitive, set_cognitive) = create_signal(
        session
            .ratings
            .as_ref()
            .map(|r| r.cognitive_load)
            .unwrap_or(0),
    );
    let (motivation, set_motivation) =
        create_signal(session.ratings.as_ref().map(|r| r.motivation).unwrap_or(0));

    let (is_submitting, set_is_submitting) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let session_id = session.id;

    let handle_save = move |_| {
        set_is_submitting.set(true);
        let t = token.get_value();

        let tag_list = tags
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let payload = UpdateSessionRequest {
            description: Some(description.get()),
            tags: Some(tag_list),
            notes: Some(notes.get()),
            ratings: Some(SessionRatings {
                mental_energy: mental.get(),
                physical_energy: physical.get(),
                cognitive_load: cognitive.get(),
                motivation: motivation.get(),
            }),
            state: None,
        };

        spawn_local(async move {
            match update_session(t, session_id, payload).await {
                Ok(_) => {
                    set_is_submitting.set(false);
                    on_updated.call(());
                }
                Err(e) => {
                    set_error_msg.set(Some(e));
                    set_is_submitting.set(false);
                }
            }
        });
    };

    let handle_delete = move |_| {
        if window()
            .confirm_with_message("Sitzung wirklich löschen?")
            .unwrap_or(false)
        {
            let t = token.get_value();
            spawn_local(async move {
                if let Ok(_) = delete_session(t, session_id).await {
                    on_delete.call(());
                }
            });
        }
    };

    view! {
        <div class="inline-editor">
            <div class="form-group">
                <label>"Beschreibung"</label>
                <input type="text"
                    prop:value=description
                    on:input=move |ev| set_description.set(event_target_value(&ev))
                />
            </div>
            <div class="form-group">
                <label>"Tags"</label>
                <input type="text"
                    prop:value=tags
                    on:input=move |ev| set_tags.set(event_target_value(&ev))
                    placeholder="rust, arbeit"
                />
            </div>
            <div class="form-group">
                <label>"Notizen"</label>
                <textarea rows="2"
                    prop:value=notes
                    on:input=move |ev| set_notes.set(event_target_value(&ev))
                ></textarea>
            </div>

            <div class="ratings-inline-grid">
                <RatingInput label="Mental" value=mental set_value=set_mental />
                <RatingInput label="Physisch" value=physical set_value=set_physical />
                <RatingInput label="Kognitiv" value=cognitive set_value=set_cognitive />
                <RatingInput label="Motivation" value=motivation set_value=set_motivation />
            </div>

            {move || error_msg.get().map(|msg| view! { <p class="error-msg">{msg}</p> })}

            <div class="editor-actions">
                <button class="delete-btn-subtle" on:click=handle_delete>"Löschen"</button>
                <button class="save-btn" on:click=handle_save disabled=is_submitting>
                    {move || if is_submitting.get() { "..." } else { "Speichern" }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn RatingInput(
    label: &'static str,
    value: ReadSignal<u8>,
    set_value: WriteSignal<u8>,
) -> impl IntoView {
    view! {
        <div class="rating-field">
            <label>{label}</label>
            <div class="rating-stars">
                {(1..=5u8).map(|i| {
                    let is_active = move || value.get() >= i;
                    view! {
                        <span
                            class="star"
                            class:active=is_active
                            on:click=move |_| set_value.set(i)
                        >
                            "★"
                        </span>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
