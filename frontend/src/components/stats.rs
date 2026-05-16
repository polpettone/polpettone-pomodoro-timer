use crate::api::API_BASE_URL;
use crate::models::Session;
use chrono::{DateTime, Utc};
use gloo_net::http::Request;
use leptos::*;
use std::collections::HashMap;

#[component]
pub fn Stats(token: String) -> impl IntoView {
    let token = store_value(token);

    // Period for stats: Last 30 days by default
    let now = Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);

    let format_date = |dt: DateTime<Utc>| dt.format("%Y-%m-%d").to_string();

    let start_date = format_date(thirty_days_ago);
    let end_date = format_date(now);

    let stats_resource = create_resource(
        move || token.get_value(),
        move |t| {
            let start = start_date.clone();
            let end = end_date.clone();
            async move {
                let url = format!(
                    "{}/sessions?start={}%2000:00:00&end={}%2023:59:59",
                    API_BASE_URL, start, end
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
            }
        },
    );

    view! {
        <div class="stats-view">
            <h2>"Statistiken (Letzte 30 Tage)"</h2>

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

                            // 1. Total Count & Time
                            let total_count = filtered.len();
                            let total_minutes: u64 = filtered.iter().map(|s| s.duration.secs / 60).sum();

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
                                // Extract date from start string "YYYY-MM-DD HH:MM:SS"
                                let date_part = s.start.split(' ').next().unwrap_or("?").to_string();
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
                                            <span class="stats-value">{if total_count > 0 { total_minutes / total_count as u64 } else { 0 }} "m"</span>
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
