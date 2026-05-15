use crate::api::{start_session, StartSessionRequest};
use leptos::*;

#[component]
pub fn StartSession(token: String, #[prop(into)] on_success: Callback<()>) -> impl IntoView {
    let (description, set_description) = create_signal("".to_string());
    let (duration, set_duration) = create_signal("25".to_string());

    let start_action = create_action(move |(desc, dur): &(String, u64)| {
        let desc = desc.clone();
        let dur = *dur;
        let token = token.clone();
        async move {
            let payload = StartSessionRequest {
                description: desc,
                duration_minutes: dur,
                tags: None,
                notes: None,
                ratings: None,
            };
            start_session(token, payload).await
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
