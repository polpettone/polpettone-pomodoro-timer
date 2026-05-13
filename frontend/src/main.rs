use leptos::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use chrono::Utc;
use web_sys::window;

const API_BASE_URL: &str = match option_env!("API_URL") {
    Some(url) => url,
    None => "http://127.0.0.1:3000",
};

fn get_token_from_storage() -> Option<String> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("ppt_token").ok().flatten())
}

fn save_token_to_storage(token: &str) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item("ppt_token", token);
    }
}

fn clear_token_from_storage() {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.remove_item("ppt_token");
    }
}

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
    let (token, set_token) = create_signal(get_token_from_storage());

    let logout = move |_| {
        clear_token_from_storage();
        set_token.set(None);
    };

    view! {
        <main class="container">
            <div class="header-flex">
                <h1>"Polpettone Pomodoro Timer"</h1>
                {move || token.get().is_some().then(|| view! {
                    <button class="logout-btn" on:click=logout>"Abmelden"</button>
                })}
            </div>
            
            {move || match token.get() {
                Some(t) => {
                    let t_active = t.clone();
                    let t_start = t.clone();
                    let t_all = t.clone();
                    let sessions = create_resource(move || t_active.clone(), |t| fetch_active_sessions(t));
                    view! {
                        <section class="card">
                            <StartSession token=t_start on_success=move |_| sessions.refetch() />
                        </section>
                        <section class="card">
                            <ActiveSessions sessions=sessions />
                        </section>
                        <section class="card">
                            <AllSessions token=t_all />
                        </section>
                    }.into_view()
                },
                None => view! {
                    <section class="card">
                        <Login on_login=move |new_token: String| {
                            save_token_to_storage(&new_token);
                            set_token.set(Some(new_token));
                        }/>
                    </section>
                }.into_view()
            }}
        </main>
    }
}

async fn fetch_active_sessions(token: String) -> Result<Vec<Session>, String> {
    let resp = Request::get(&format!("{}/sessions/active", API_BASE_URL))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        if resp.status() == 401 {
            return Err("Nicht autorisiert. Bitte erneut anmelden.".to_string());
        }
        return Err(format!("Fehler beim Laden: {}", resp.status()));
    }

    resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_all_sessions(token: String) -> Result<Vec<Session>, String> {
    let resp = Request::get(&format!("{}/sessions?start=2000-01-01%2000:00:00&end=2099-12-31%2023:59:59", API_BASE_URL))
        .header("Authorization", &format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !resp.ok() {
        if resp.status() == 401 {
            return Err("Nicht autorisiert.".to_string());
        }
        return Err(format!("Fehler beim Laden der Historie: {}", resp.status()));
    }

    let mut sessions = resp.json::<Vec<Session>>()
        .await
        .map_err(|e| e.to_string())?;
    
    sessions.sort_by(|a, b| b.start.cmp(&a.start));
    
    Ok(sessions)
}

#[component]
fn ActiveSessions(sessions: Resource<String, Result<Vec<Session>, String>>) -> impl IntoView {
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
fn AllSessions(token: String) -> impl IntoView {
    let (is_open, set_is_open) = create_signal(false);
    let sessions = create_resource(move || (is_open.get(), token.clone()), |(open, t)| async move {
        if open {
            fetch_all_sessions(t).await
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
fn StartSession(token: String, #[prop(into)] on_success: Callback<()>) -> impl IntoView {
    let (description, set_description) = create_signal("".to_string());
    let (duration, set_duration) = create_signal("25".to_string());

    let start_action = create_action(move |(desc, dur): &(String, u64)| {
        let desc = desc.clone();
        let dur = *dur;
        let token = token.clone();
        async move {
            let payload = serde_json::json!({
                "description": desc,
                "duration_minutes": dur,
            });

            Request::post(&format!("{}/sessions/start", API_BASE_URL))
                .header("Authorization", &format!("Bearer {}", token))
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

#[component]
fn Login(#[prop(into)] on_login: Callback<String>) -> impl IntoView {
    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());

    let login_action = create_action(move |(u, p): &(String, String)| {
        let u = u.clone();
        let p = p.clone();
        async move {
            let payload = serde_json::json!({
                "username": u,
                "password": p,
            });

            let resp = Request::post(&format!("{}/login", API_BASE_URL))
                .json(&payload)
                .map_err(|e| e.to_string())?
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !resp.ok() {
                return Err("Login fehlgeschlagen. Überprüfen Sie Ihre Zugangsdaten.".to_string());
            }

            let data = resp.json::<serde_json::Value>()
                .await
                .map_err(|e| e.to_string())?;
            
            let token = data["token"].as_str()
                .ok_or_else(|| "Kein Token in der Antwort".to_string())?;
            
            Ok::<String, String>(token.to_string())
        }
    });

    create_effect(move |_| {
        if let Some(Ok(token)) = login_action.value().get() {
            on_login.call(token);
        }
    });

    view! {
        <div class="login-form">
            <h2>"Anmelden"</h2>
            <div class="flex flex-col gap-2">
                <input 
                    type="text" 
                    placeholder="Benutzername"
                    on:input=move |ev| set_username.set(event_target_value(&ev))
                    prop:value=username
                />
                <input 
                    type="password" 
                    placeholder="Passwort"
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            login_action.dispatch((username.get(), password.get()));
                        }
                    }
                    prop:value=password
                />
                <button 
                    on:click=move |_| login_action.dispatch((username.get(), password.get()))
                    disabled=move || login_action.pending().get()
                >
                    "Anmelden"
                </button>
            </div>
            {move || {
                if let Some(res) = login_action.value().get() {
                    if let Err(e) = res {
                        view! { <p class="status-msg" style="color: var(--accent-color)">{e}</p> }.into_view()
                    } else {
                        view! { <div class="status-msg" /> }.into_view()
                    }
                } else {
                    view! { <div class="status-msg" /> }.into_view()
                }
            }}
            <p class="hint">"Tipp: admin / admin"</p>
        </div>
    }
}
