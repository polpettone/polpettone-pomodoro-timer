use leptos::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use chrono::Utc;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Session {
    pub description: String,
    pub duration: Duration,
    pub start: String,
    pub state: String,
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    let sessions = create_resource(|| (), |_| fetch_active_sessions());

    view! {
        <main class="container">
            <h1>"Polpettone Pomodoro Timer"</h1>
            <section class="card">
                <StartSession on_success=move |_| sessions.refetch() />
            </section>
            <section class="card">
                <ActiveSessions sessions=sessions />
            </section>
            <section class="card">
                <AllSessions />
            </section>
        </main>
    }
}

async fn fetch_active_sessions() -> Result<Vec<Session>, String> {
    let resp = Request::get("http://127.0.0.1:3000/sessions/active")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        return Err(format!("Fehler beim Laden: {}", resp.status()));
    }

    resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_all_sessions() -> Result<Vec<Session>, String> {
    let resp = Request::get("http://127.0.0.1:3000/sessions?start=2000-01-01%2000:00:00&end=2099-12-31%2023:59:59")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        return Err(format!("Fehler beim Laden der Historie: {}", resp.status()));
    }

    let mut sessions = resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())?;
    
    sessions.sort_by(|a, b| b.start.cmp(&a.start));
    
    Ok(sessions)
}

#[component]
fn ActiveSessions(sessions: Resource<(), Result<Vec<Session>, String>>) -> impl IntoView {
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
                                    <ul>
                                        {data.into_iter().map(|s| {
                                            view! {
                                                <li>
                                                    <strong>{s.description.clone()}</strong>
                                                    " - " <Timer session=s />
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
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

#[component]
fn Timer(session: Session) -> impl IntoView {
    let (now, set_now) = create_signal(Utc::now());

    // Update die aktuelle Zeit jede Sekunde
    set_interval(
        move || {
            set_now.set(Utc::now());
        },
        std::time::Duration::from_secs(1),
    );

    let remaining = move || {
        // Parse das Startdatum vom Backend
        let start = chrono::NaiveDateTime::parse_from_str(&session.start, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .unwrap_or_else(|_| Utc::now());
        
        let duration = chrono::Duration::seconds(session.duration.secs as i64);
        let end = start + duration;
        let diff = end - now.get();
        
        if diff.num_seconds() <= 0 {
            "00:00 (Beendet)".to_string()
        } else {
            let minutes = diff.num_minutes();
            let seconds = diff.num_seconds() % 60;
            format!("{:02}:{:02}", minutes, seconds)
        }
    };

    view! {
        <span class="timer-display">{remaining}</span>
    }
}

#[component]
fn AllSessions() -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let sessions = create_resource(move || is_open.get(), |open| async move {
        if open {
            fetch_all_sessions().await
        } else {
            Ok(vec![])
        }
    });

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
                                        <div class="table-container">
                                            <table>
                                                <thead>
                                                    <tr>
                                                        <th>"Start"</th>
                                                        <th>"Beschreibung"</th>
                                                        <th>"Dauer"</th>
                                                        <th>"Status"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {data.into_iter().map(|s| {
                                                        view! {
                                                            <tr>
                                                                <td class="text-nowrap">{s.start}</td>
                                                                <td>{s.description}</td>
                                                                <td class="text-nowrap">{s.duration.secs / 60} " Min"</td>
                                                                <td><span class=format!("state-tag {}", s.state.to_lowercase())>{s.state}</span></td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
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
fn StartSession(#[prop(into)] on_success: Callback<()>) -> impl IntoView {
    let (description, set_description) = create_signal("".to_string());
    let (duration, set_duration) = create_signal("25".to_string());

    let start_action = create_action(|(desc, dur): &(String, u64)| {
        let desc = desc.clone();
        let dur = *dur;
        async move {
            let payload = serde_json::json!({
                "description": desc,
                "duration_minutes": dur,
            });

            Request::post("http://127.0.0.1:3000/sessions/start")
                .json(&payload)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;
            Ok::<(), String>(())
        }
    });

    create_effect(move |_| {
        if let Some(Ok(_)) = start_action.value().get() {
            set_description.set("".to_string());
            on_success.call(());
        }
    });

    view! {
        <div>
            <h2>"Neue Sitzung starten"</h2>
            <div class="flex justify-center items-center">
                <input 
                    type="text" 
                    placeholder="Beschreibung"
                    on:input=move |ev| set_description.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            start_action.dispatch((description.get(), duration.get().parse().unwrap_or(25)));
                        }
                    }
                    prop:value=description
                />
                <input 
                    type="text" 
                    inputmode="numeric"
                    style="width: 80px"
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                        set_duration.set(filtered);
                    }
                    prop:value=duration
                />
                <button 
                    on:click=move |_| start_action.dispatch((description.get(), duration.get().parse().unwrap_or(25)))
                    disabled=move || start_action.pending().get()
                >
                    "Start"
                </button>
            </div>
            {move || {
                if let Some(res) = start_action.value().get() {
                    match res {
                        Ok(_) => view! { <p class="status-msg" style="color: #4caf50">"Sitzung gestartet!"</p> }.into_view(),
                        Err(e) => view! { <p class="status-msg" style="color: var(--accent-color)">{e}</p> }.into_view(),
                    }
                } else {
                    view! { <div class="status-msg" /> }.into_view()
                }
            }}
        </div>
    }
}
