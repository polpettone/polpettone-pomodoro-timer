use crate::api::API_BASE_URL;
use crate::models::Session;
use chrono::{Datelike, Duration, Utc};
use gloo_net::http::Request;
use leptos::*;
use std::collections::HashMap;

#[component]
pub fn Stats(token: String) -> impl IntoView {
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

    let stats_resource = create_resource(
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
                return Err(format!(
                    "Fehler beim Laden der Statistiken: {}",
                    resp.status()
                ));
            }

            let data = resp
                .json::<Vec<Session>>()
                .await
                .map_err(|e| e.to_string())?;

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
        <div class="stats-view">
            <h2>"Statistiken"</h2>

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

            <Transition fallback=move || view! { <p class="status-msg">"Berechne Statistiken..."</p> }>
                {move || {
                    stats_resource.get().map(|res| match res {
                        Ok(sessions) => {
                            let filtered: Vec<_> = sessions.into_iter()
                                .filter(|s| s.state != "Deleted")
                                .collect();

                            if filtered.is_empty() {
                                return view! { <p class="status-msg">"Keine Daten für diesen Zeitraum vorhanden."</p> }.into_view();
                            }

                            let (total_count, total_minutes, avg_minutes) = calculate_stats(&filtered);

                            // 2. Count by Description
                            let mut desc_counts: HashMap<String, usize> = HashMap::new();
                            for s in &filtered {
                                *desc_counts.entry(s.description.clone()).or_insert(0) += 1;
                            }
                            let mut desc_counts_vec: Vec<_> = desc_counts.into_iter().collect();
                            desc_counts_vec.sort_by(|a, b| b.1.cmp(&a.1));

                            // 3. Sessions per Day
                            let mut day_counts: HashMap<String, usize> = HashMap::new();
                            for s in &filtered {
                                let date_part = extract_date_from_start(&s.start);
                                *day_counts.entry(date_part).or_insert(0) += 1;
                            }
                            let mut day_counts_vec: Vec<_> = day_counts.into_iter().collect();
                            day_counts_vec.sort_by(|a, b| a.0.cmp(&b.0));

                            view! {
                                <div class="stats-grid">
                                    <div class="stats-card">
                                        <h3>"Übersicht"</h3>
                                        <div class="stats-row">
                                            <span>"Gesamtanzahl:"</span>
                                            <span class="stats-value">{total_count}</span>
                                        </div>
                                        <div class="stats-row">
                                            <span>"Gesamtzeit:"</span>
                                            <span class="stats-value">{total_minutes / 60} "h " {total_minutes % 60} "m"</span>
                                        </div>
                                        <div class="stats-row">
                                            <span>"Ø Dauer:"</span>
                                            <span class="stats-value">{avg_minutes} "m"</span>
                                        </div>
                                    </div>

                                    <div class="stats-card">
                                        <h3>"Top Aktivitäten"</h3>
                                        <div class="stats-list-scroll">
                                            {desc_counts_vec.into_iter().take(5).map(|(desc, count)| {
                                                view! {
                                                    <div class="stats-row">
                                                        <span class="stats-desc-label">{desc}</span>
                                                        <span class="stats-value">{count}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    <div class="stats-card full-width">
                                        <h3>"Verlauf (Sessions pro Tag)"</h3>
                                        <div class="stats-timeline">
                                            {day_counts_vec.into_iter().map(|(date, count)| {
                                                let height = (count as f32 * 20.0).min(100.0);
                                                let day = date.split('-').last().unwrap_or("").to_string();
                                                view! {
                                                    <div class="timeline-item">
                                                        <div class="timeline-bar" style=format!("height: {}px", height)>
                                                            <span class="timeline-count">{count}</span>
                                                        </div>
                                                        <span class="timeline-date">{day}</span>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
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

fn calculate_stats(sessions: &[Session]) -> (usize, u64, u64) {
    let total_count = sessions.len();
    let total_minutes: u64 = sessions.iter().map(|s| s.duration.secs / 60).sum();
    let avg_minutes = if total_count > 0 {
        total_minutes / total_count as u64
    } else {
        0
    };
    (total_count, total_minutes, avg_minutes)
}

fn extract_date_from_start(start: &str) -> String {
    start.split(' ').next().unwrap_or("?").to_string()
}
