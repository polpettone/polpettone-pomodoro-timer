use crate::api::API_BASE_URL;
use crate::models::Session;
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
                                        let ratings = session.ratings.clone().unwrap_or_default();
                                        view! {
                                            <div class="session-card evaluation-card card">
                                                <div class="evaluation-card-header">
                                                    <span class="evaluation-date">{session.start.clone()}</span>
                                                    <span class="evaluation-duration">{session.duration.secs / 60} "m"</span>
                                                </div>
                                                <h3 class="evaluation-title">{session.description.clone()}</h3>

                                                {if !session.tags.is_empty() {
                                                    view! {
                                                        <div class="evaluation-tags">
                                                            {session.tags.iter().map(|tag| {
                                                                view! { <span class="tag-label">{tag}</span> }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! { <div/> }.into_view()
                                                }}

                                                {if !session.notes.is_empty() {
                                                    view! {
                                                        <div class="evaluation-notes">
                                                            <strong>"Notizen:"</strong>
                                                            <p>{session.notes.clone()}</p>
                                                        </div>
                                                    }.into_view()
                                                } else {
                                                    view! { <div/> }.into_view()
                                                }}

                                                <div class="evaluation-ratings">
                                                    <div class="rating-item">
                                                        <span class="rating-label">"🧠 Mental:"</span>
                                                        <span class="rating-value">{ratings.mental_energy}</span>
                                                    </div>
                                                    <div class="rating-item">
                                                        <span class="rating-label">"💪 Physisch:"</span>
                                                        <span class="rating-value">{ratings.physical_energy}</span>
                                                    </div>
                                                    <div class="rating-item">
                                                        <span class="rating-label">"⚙️ Kognitiv:"</span>
                                                        <span class="rating-value">{ratings.cognitive_load}</span>
                                                    </div>
                                                    <div class="rating-item">
                                                        <span class="rating-label">"🔥 Motivation:"</span>
                                                        <span class="rating-value">{ratings.motivation}</span>
                                                    </div>
                                                </div>
                                            </div>
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
