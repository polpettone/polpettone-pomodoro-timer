use crate::api::{register, API_BASE_URL};
use gloo_net::http::Request;
use leptos::*;

#[component]
pub fn Login(#[prop(into)] on_login: Callback<String>) -> impl IntoView {
    let (is_register_mode, set_is_register_mode) = create_signal(false);
    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (confirm_password, set_confirm_password) = create_signal("".to_string());
    let (error_msg, set_error_msg) = create_signal(None::<String>);
    let (success_msg, set_success_msg) = create_signal(None::<String>);

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
                let error_text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Login fehlgeschlagen".to_string());
                return Err(error_text);
            }

            let data = resp
                .json::<serde_json::Value>()
                .await
                .map_err(|e| e.to_string())?;

            let token = data["token"]
                .as_str()
                .ok_or_else(|| "Kein Token in der Antwort".to_string())?;

            Ok::<String, String>(token.to_string())
        }
    });

    let register_action = create_action(move |(u, p): &(String, String)| {
        let u = u.clone();
        let p = p.clone();
        async move { register(u, p).await }
    });

    create_effect(move |_| {
        if let Some(Ok(token)) = login_action.value().get() {
            on_login.call(token);
        }
    });

    create_effect(move |_| {
        if let Some(res) = register_action.value().get() {
            match res {
                Ok(_) => {
                    set_success_msg.set(Some(
                        "Registrierung erfolgreich! Bitte melden Sie sich an.".to_string(),
                    ));
                    set_error_msg.set(None);
                    set_is_register_mode.set(false);
                }
                Err(e) => {
                    set_error_msg.set(Some(e));
                }
            }
        }
    });

    let submit_logic = move || {
        set_error_msg.set(None);
        set_success_msg.set(None);
        let u = username.get();
        let p = password.get();

        if u.is_empty() || p.is_empty() {
            set_error_msg.set(Some("Bitte alle Felder ausfüllen.".to_string()));
            return;
        }

        if is_register_mode.get() {
            if p != confirm_password.get() {
                set_error_msg.set(Some("Passwörter stimmen nicht überein.".to_string()));
                return;
            }
            register_action.dispatch((u, p));
        } else {
            login_action.dispatch((u, p));
        }
    };

    view! {
        <div class="login-form">
            <h2>{move || if is_register_mode.get() { "Registrieren" } else { "Anmelden" }}</h2>
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
                        if ev.key() == "Enter" && !is_register_mode.get() {
                            submit_logic();
                        }
                    }
                    prop:value=password
                />

                <Show when=move || is_register_mode.get()>
                    <input
                        type="password"
                        placeholder="Passwort bestätigen"
                        on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                submit_logic();
                            }
                        }
                        prop:value=confirm_password
                    />
                </Show>

                <button
                    on:click=move |_| submit_logic()
                    disabled=move || login_action.pending().get() || register_action.pending().get()
                >
                    {move || if is_register_mode.get() { "Registrieren" } else { "Anmelden" }}
                </button>

                <div class="auth-switch">
                    {move || if is_register_mode.get() {
                        view! {
                            <span>"Bereits ein Konto? "</span>
                            <a href="#" on:click=move |e| {
                                e.prevent_default();
                                set_is_register_mode.set(false);
                                set_error_msg.set(None);
                                set_success_msg.set(None);
                            }>"Hier anmelden"</a>
                        }.into_view()
                    } else {
                        view! {
                            <span>"Noch kein Konto? "</span>
                            <a href="#" on:click=move |e| {
                                e.prevent_default();
                                set_is_register_mode.set(true);
                                set_error_msg.set(None);
                                set_success_msg.set(None);
                            }>"Hier registrieren"</a>
                        }.into_view()
                    }}
                </div>
            </div>

            {move || {
                let err = error_msg.get().or_else(|| {
                    login_action.value().get().and_then(|res| res.err())
                });

                if let Some(e) = err {
                    view! { <p class="status-msg" style="color: var(--accent-color)">{e}</p> }.into_view()
                } else if let Some(s) = success_msg.get() {
                    view! { <p class="status-msg" style="color: #4caf50">{s}</p> }.into_view()
                } else {
                    view! { <div class="status-msg" /> }.into_view()
                }
            }}
        </div>
    }
}
