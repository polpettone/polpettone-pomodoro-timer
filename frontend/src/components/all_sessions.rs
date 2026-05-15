use crate::api::{
    delete_session, start_session, update_session, StartSessionRequest, UpdateSessionRequest,
    API_BASE_URL,
};
use crate::models::{Session, SessionRatings};
use chrono::{Datelike, Duration, Utc};
use gloo_net::http::Request;
use leptos::*;

#[component]
pub fn AllSessions(token: String, #[prop(into)] on_update: Callback<()>) -> impl IntoView {
    let token = store_value(token);
    let (is_open, set_is_open) = create_signal(false);
    let (expanded_id, set_expanded_id) = create_signal(None::<uuid::Uuid>);

    // Filter signals
    let (search_query, set_search_query) = create_signal("".to_string());

    let now = Utc::now();
    let format_date =
        |dt: chrono::DateTime<Utc>| format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day());

    let today_str = format_date(now);
    let month_ago_str = format_date(now - Duration::days(30));

    let (start_date, set_start_date) = create_signal(month_ago_str.clone());
    let (end_date, set_end_date) = create_signal(today_str.clone());
    let (show_deleted, set_show_deleted) = create_signal(false);

    let sessions = create_resource(
        move || {
            (
                is_open.get(),
                token.get_value(),
                search_query.get(),
                start_date.get(),
                end_date.get(),
            )
        },
        |(open, t, query, start, end)| async move {
            if !open {
                return Ok(vec![]);
            }

            let url = format!(
                "{}/sessions?start={}%2000:00:00&end={}%2023:59:59&query={}",
                API_BASE_URL,
                start,
                end,
                query.replace(" ", "%20")
            );

            let resp = Request::get(&url)
                .header("Authorization", &format!("Bearer {}", t))
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.ok() {
                if resp.status() == 401 {
                    return Err("Nicht autorisiert.".to_string());
                }
                return Err(format!("Fehler beim Laden der Historie: {}", resp.status()));
            }

            let mut data = resp
                .json::<Vec<Session>>()
                .await
                .map_err(|e| e.to_string())?;

            data.sort_by(|a, b| b.start.cmp(&a.start));

            Ok(data)
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

    // Quick filter handlers
    let set_filter_today = move |_| {
        let d = format_date(Utc::now());
        set_start_date.set(d.clone());
        set_end_date.set(d);
    };

    let set_filter_yesterday = move |_| {
        let d = format_date(Utc::now() - Duration::days(1));
        set_start_date.set(d.clone());
        set_end_date.set(d);
    };

    let set_filter_last_week = move |_| {
        set_start_date.set(format_date(Utc::now() - Duration::days(7)));
        set_end_date.set(format_date(Utc::now()));
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
                <div class="filters-container card">
                    <div class="flex gap-2 items-center wrap mb-1">
                        <input
                            type="text"
                            placeholder="Suchen..."
                            class="filter-search"
                            prop:value=search_query
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                        <div class="flex items-center gap-1">
                            <input
                                type="date"
                                prop:value=start_date
                                on:change=move |ev| set_start_date.set(event_target_value(&ev))
                            />
                            <span>" bis "</span>
                            <input
                                type="date"
                                prop:value=end_date
                                on:change=move |ev| set_end_date.set(event_target_value(&ev))
                            />
                        </div>
                        <label class="checkbox-label">
                            <input
                                type="checkbox"
                                prop:checked=show_deleted
                                on:change=move |ev| set_show_deleted.set(event_target_checked(&ev))
                            />
                            " Gelöschte"
                        </label>
                    </div>
                    <div class="flex gap-1 wrap">
                        <button class="filter-preset-btn" on:click=set_filter_today>"Heute"</button>
                        <button class="filter-preset-btn" on:click=set_filter_yesterday>"Gestern"</button>
                        <button class="filter-preset-btn" on:click=set_filter_last_week>"Letzte 7 Tage"</button>
                        <button class="filter-preset-btn" on:click={
                            let m = month_ago_str.clone();
                            let t = today_str.clone();
                            move |_| {
                                set_start_date.set(m.clone());
                                set_end_date.set(t.clone());
                            }
                        }>"Letzte 30 Tage"</button>
                    </div>
                </div>

                <Transition fallback=move || view! { <p class="status-msg">"Lade Historie..."</p> }>
                    {move || {
                        sessions.get().map(|res| match res {
                            Ok(data) => {
                                let filtered_data: Vec<_> = data.into_iter()
                                    .filter(|s| show_deleted.get() || s.state != "Deleted")
                                    .collect();

                                if filtered_data.is_empty() {
                                    view! { <p class="status-msg">"Keine Sitzungen gefunden."</p> }.into_view()
                                } else {
                                    view! {
                                        <div class="sessions-list">
                                            {filtered_data.into_iter().map(|s| {
                                                let s_id = s.id;
                                                let s_stored = store_value(s);
                                                let is_expanded = move || expanded_id.get() == Some(s_id);

                                                let on_quick_clone = {
                                                    let t = token.get_value();
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
                                                                on_update.call(());
                                                            }
                                                        });
                                                    }
                                                };

                                                let on_quick_delete = {
                                                    let t = token.get_value();
                                                    move |ev: leptos::ev::MouseEvent| {
                                                        ev.stop_propagation();
                                                        if window().confirm_with_message("Sitzung wirklich löschen?").unwrap_or(false) {
                                                            let t = t.clone();
                                                            spawn_local(async move {
                                                                if let Ok(_) = delete_session(t, s_id).await {
                                                                    sessions.refetch();
                                                                    on_update.call(());
                                                                }
                                                            });
                                                        }
                                                    }
                                                };

                                                let on_summary_click = move |_| toggle_expand(s_id);
                                                let on_summary_keydown = move |ev: leptos::ev::KeyboardEvent| {
                                                    if ev.key() == "Enter" || ev.key() == " " {
                                                        ev.prevent_default();
                                                        toggle_expand(s_id);
                                                    }
                                                };

                                                view! {
                                                    <div class="session-item-container" class:expanded=is_expanded>
                                                        <div
                                                            class="session-summary"
                                                            role="button"
                                                            tabindex="0"
                                                            on:click=on_summary_click
                                                            on:keydown=on_summary_keydown
                                                        >
                                                            <div class="session-main-info">
                                                                <span class="session-time">{move || s_stored.get_value().start}</span>
                                                                <span class="session-desc">{move || s_stored.get_value().description}</span>
                                                            </div>
                                                            <div class="session-meta-info">
                                                                <button
                                                                    class="quick-delete-btn"
                                                                    title="Löschen"
                                                                    on:click=on_quick_delete
                                                                >
                                                                    "🗑"
                                                                </button>
                                                                <button
                                                                    class="quick-clone-btn"
                                                                    title="Klonen & Starten"
                                                                    on:click=on_quick_clone
                                                                >
                                                                    "▶"
                                                                </button>
                                                                <span class="session-duration">{move || s_stored.get_value().duration.secs / 60} "m"</span>
                                                                <span class=move || format!("state-tag {}", s_stored.get_value().state.to_lowercase())>
                                                                    {move || s_stored.get_value().state}
                                                                </span>
                                                                <span class="expand-icon">{move || if is_expanded() { "▲" } else { "▼" }}</span>
                                                            </div>
                                                        </div>

                                                        <Show when=is_expanded>
                                                            <SessionEditor
                                                                token=token.get_value()
                                                                session=s_stored.get_value()
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
pub fn SessionEditor(
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

    let handle_clone_start = move |_| {
        set_is_submitting.set(true);
        let t = token.get_value();

        let tag_list = tags
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let payload = StartSessionRequest {
            description: description.get(),
            duration_minutes: session.duration.secs / 60,
            tags: Some(tag_list),
            notes: Some(notes.get()),
            ratings: Some(SessionRatings {
                mental_energy: mental.get(),
                physical_energy: physical.get(),
                cognitive_load: cognitive.get(),
                motivation: motivation.get(),
            }),
        };

        spawn_local(async move {
            match start_session(t, payload).await {
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
                <div class="flex gap-2">
                    <button class="delete-btn-subtle" on:click=handle_delete>
                        "Löschen"
                    </button>
                    <button
                        class="edit-btn"
                        on:click=handle_clone_start
                        disabled=is_submitting
                    >
                        "Klonen & Start"
                    </button>
                </div>
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
                            role="button"
                            tabindex="0"
                            on:click=move |_| set_value.set(i)
                            on:keydown={move |ev: leptos::ev::KeyboardEvent| {
                                if ev.key() == "Enter" || ev.key() == " " {
                                    ev.prevent_default();
                                    set_value.set(i);
                                }
                            }}
                        >
                            "★"
                        </span>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
