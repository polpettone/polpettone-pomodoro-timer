use crate::api::API_BASE_URL;
use gloo_net::http::Request;
use leptos::*;

#[component]
pub fn Login(#[prop(into)] on_login: Callback<String>) -> impl IntoView {
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
        </div>
    }
}
