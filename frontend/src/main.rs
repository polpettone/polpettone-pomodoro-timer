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
fn StartSession(#[prop(into)] on_success: Callback<()>) -> impl IntoView {
    let (description, set_description) = create_signal("".to_string());
    let (duration, set_duration) = create_signal(25);

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
                            start_action.dispatch((description.get(), duration.get()));
                        }
                    }
                    prop:value=description
                />
                <input 
                    type="number" 
                    style="width: 80px"
                    on:input=move |ev| set_duration.set(event_target_value(&ev).parse().unwrap_or(25))
                    prop:value=duration
                />
                <button 
                    on:click=move |_| start_action.dispatch((description.get(), duration.get()))
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
