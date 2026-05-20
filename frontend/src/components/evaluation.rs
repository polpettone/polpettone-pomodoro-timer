use crate::api::{update_session, UpdateSessionRequest, API_BASE_URL};
use crate::models::{Session, SessionRatings};
use chrono::{Datelike, Duration, Utc};
use gloo_net::http::Request;
use leptos::*;

#[component]
pub fn Evaluation(token: String) -> impl IntoView {
    let token = store_value(token);

    let now = Utc::now();
    let format_date =
        |dt: chrono::DateTime<Utc>| format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day());

    let today_str = format_date(now);
    let month_ago_str = format_date(now - Duration::days(30));

    // Filter signals
    let (start_date, set_start_date) = create_signal(month_ago_str.clone());
    let (end_date, set_end_date) = create_signal(today_str.clone());
    let (search_query, set_search_query) = create_signal("".to_string());

    let sessions_resource = create_resource(
        move || {
            (
                token.get_value(),
                start_date.get(),
                end_date.get(),
                search_query.get(),
            )
        },
        |(t, start, end, query)| async move {
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
                return Err(format!("Fehler beim Laden der Sessions: {}", resp.status()));
            }

            let mut data = resp
                .json::<Vec<Session>>()
                .await
                .map_err(|e| e.to_string())?;

            data.sort_by(|a, b| b.start.cmp(&a.start));

            Ok(data)
        },
    );

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

    let set_filter_all = move |_| {
        set_start_date.set("1970-01-01".to_string());
        set_end_date.set(format_date(Utc::now()));
    };

    view! {
        <div class="evaluation-view">
            <h2>"Auswertung"</h2>

            <div class="filters-container card mb-1">
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
                </div>
                <div class="flex gap-1 wrap">
                    <button class="filter-preset-btn" on:click=set_filter_all>"Alle"</button>
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

            <Transition fallback=move || view! { <p class="status-msg">"Lade Sessions..."</p> }>
                {move || {
                    sessions_resource.get().map(|res| match res {
                        Ok(sessions) => {
                            let filtered: Vec<_> = sessions.into_iter()
                                .filter(|s| s.state != "Deleted")
                                .collect();

                            if filtered.is_empty() {
                                return view! { <p class="status-msg">"Keine Daten für diesen Zeitraum vorhanden."</p> }.into_view();
                            }

                            view! {
                                <div class="evaluation-grid">
                                    {filtered.into_iter().map(|session| {
                                        view! {
                                            <EvaluationCard
                                                token=token.get_value()
                                                session=session
                                                on_updated=move |_| { sessions_resource.refetch(); }
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        },
                        Err(e) => view! { <p class="error-msg">{e}</p> }.into_view()
                    })
                }}
            </Transition>
        </div>
    }
}

#[component]
fn EvaluationCard(
    token: String,
    session: Session,
    #[prop(into)] on_updated: Callback<()>,
) -> impl IntoView {
    let (description, set_description) = create_signal(session.description.clone());
    let (tags_str, set_tags_str) = create_signal(session.tags.join(", "));
    let (notes, set_notes) = create_signal(session.notes.clone());

    let ratings = session.ratings.unwrap_or_default();
    let (mental, set_mental) = create_signal(ratings.mental_energy);
    let (physical, set_physical) = create_signal(ratings.physical_energy);
    let (cognitive, set_cognitive) = create_signal(ratings.cognitive_load);
    let (motivation, set_motivation) = create_signal(ratings.motivation);

    let (is_saving, set_is_saving) = create_signal(false);

    let handle_save = move |_| {
        set_is_saving.set(true);
        let t = token.clone();
        let id = session.id;

        let tag_list: Vec<String> = tags_str
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
            if let Ok(_) = update_session(t, id, payload).await {
                set_is_saving.set(false);
                on_updated.call(());
            } else {
                set_is_saving.set(false);
            }
        });
    };

    view! {
        <div class="session-card evaluation-card card">
            <div class="evaluation-card-header">
                <span class="evaluation-date">{session.start.clone()}</span>
                <span class="evaluation-duration">{session.duration.secs / 60} "m"</span>
            </div>

            <input
                type="text"
                class="evaluation-title-input"
                prop:value=description
                on:input=move |ev| set_description.set(event_target_value(&ev))
            />

            <div class="evaluation-field">
                <label>"Tags"</label>
                <input
                    type="text"
                    class="evaluation-tags-input"
                    prop:value=tags_str
                    on:input=move |ev| set_tags_str.set(event_target_value(&ev))
                    placeholder="tag1, tag2..."
                />
            </div>

            <div class="evaluation-field">
                <label>"Notizen"</label>
                <textarea
                    class="evaluation-notes-input"
                    rows="2"
                    prop:value=notes
                    on:input=move |ev| set_notes.set(event_target_value(&ev))
                ></textarea>
            </div>

            <div class="evaluation-ratings">
                <RatingControl label="🧠" value=mental set_value=set_mental />
                <RatingControl label="💪" value=physical set_value=set_physical />
                <RatingControl label="⚙️" value=cognitive set_value=set_cognitive />
                <RatingControl label="🔥" value=motivation set_value=set_motivation />
            </div>

            <button
                class="save-btn evaluation-save-btn"
                on:click=handle_save
                disabled=is_saving
            >
                {move || if is_saving.get() { "..." } else { "Speichern" }}
            </button>
        </div>
    }
}

#[component]
fn RatingControl(
    label: &'static str,
    value: ReadSignal<u8>,
    set_value: WriteSignal<u8>,
) -> impl IntoView {
    let inc = move |_| {
        set_value.update(|v| {
            if *v < 5 {
                *v += 1
            }
        })
    };
    let dec = move |_| {
        set_value.update(|v| {
            if *v > 0 {
                *v -= 1
            }
        })
    };

    view! {
        <div class="rating-item">
            <span class="rating-label">{label}</span>
            <div class="rating-controls">
                <button class="rating-btn" on:click=dec>"-"</button>
                <span class="rating-value">{value}</span>
                <button class="rating-btn" on:click=inc>"+"</button>
            </div>
        </div>
    }
}
