use crate::api::{update_session, UpdateSessionRequest};
use crate::models::{Session, SessionRatings};
use leptos::*;

#[component]
pub fn EditSession(
    token: String,
    session: Session,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_updated: Callback<()>,
) -> impl IntoView {
    let (description, set_description) = create_signal(session.description.clone());
    let (notes, set_notes) = create_signal(session.notes.clone());
    let (tags, set_tags) = create_signal(session.tags.join(", "));

    // Ratings signals
    let (mental, set_mental) = create_signal(
        session
            .ratings
            .as_ref()
            .map(|r| r.mental_energy)
            .unwrap_or(0),
    );
    let (physical, set_physical) = create_signal(
        session
            .ratings
            .as_ref()
            .map(|r| r.physical_energy)
            .unwrap_or(0),
    );
    let (cognitive, set_cognitive) = create_signal(
        session
            .ratings
            .as_ref()
            .map(|r| r.cognitive_load)
            .unwrap_or(0),
    );
    let (motivation, set_motivation) =
        create_signal(session.ratings.as_ref().map(|r| r.motivation).unwrap_or(0));

    let (is_submitting, set_is_submitting) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal(None::<String>);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_submitting.set(true);
        set_error_msg.set(None);

        let token_clone = token.clone();
        let session_id = session.id;

        let tag_list = tags
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let ratings = SessionRatings {
            mental_energy: mental.get(),
            physical_energy: physical.get(),
            cognitive_load: cognitive.get(),
            motivation: motivation.get(),
        };

        let payload = UpdateSessionRequest {
            description: Some(description.get()),
            tags: Some(tag_list),
            notes: Some(notes.get()),
            ratings: Some(ratings),
            state: None, // We don't change the state here
        };

        spawn_local(async move {
            match update_session(token_clone, session_id, payload).await {
                Ok(_) => {
                    on_updated.call(());
                    on_close.call(());
                }
                Err(e) => {
                    set_error_msg.set(Some(e));
                    set_is_submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-content card">
                <div class="modal-header">
                    <h3>"Sitzung bearbeiten"</h3>
                    <button class="close-btn" on:click=move |_| on_close.call(())>"&times;"</button>
                </div>

                <form on:submit=handle_submit>
                    <div class="form-group">
                        <label>"Beschreibung"</label>
                        <input
                            type="text"
                            prop:value=description
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            required
                        />
                    </div>

                    <div class="form-group">
                        <label>"Tags (kommagetrennt)"</label>
                        <input
                            type="text"
                            prop:value=tags
                            on:input=move |ev| set_tags.set(event_target_value(&ev))
                            placeholder="rust, arbeit, fokus"
                        />
                    </div>

                    <div class="form-group">
                        <label>"Notizen"</label>
                        <textarea
                            rows="3"
                            prop:value=notes
                            on:input=move |ev| set_notes.set(event_target_value(&ev))
                        ></textarea>
                    </div>

                    <div class="ratings-grid">
                        <div class="form-group">
                            <label>"Mentale Energie (1-5)"</label>
                            <input type="number" min="0" max="5"
                                prop:value=mental
                                on:input=move |ev| set_mental.set(event_target_value(&ev).parse().unwrap_or(0))
                            />
                        </div>
                        <div class="form-group">
                            <label>"Physische Energie (1-5)"</label>
                            <input type="number" min="0" max="5"
                                prop:value=physical
                                on:input=move |ev| set_physical.set(event_target_value(&ev).parse().unwrap_or(0))
                            />
                        </div>
                        <div class="form-group">
                            <label>"Kognitive Last (1-5)"</label>
                            <input type="number" min="0" max="5"
                                prop:value=cognitive
                                on:input=move |ev| set_cognitive.set(event_target_value(&ev).parse().unwrap_or(0))
                            />
                        </div>
                        <div class="form-group">
                            <label>"Motivation (1-5)"</label>
                            <input type="number" min="0" max="5"
                                prop:value=motivation
                                on:input=move |ev| set_motivation.set(event_target_value(&ev).parse().unwrap_or(0))
                            />
                        </div>
                    </div>

                    {move || error_msg.get().map(|msg| view! { <p class="error-msg">{msg}</p> })}

                    <div class="form-actions">
                        <button type="button" class="secondary-btn" on:click=move |_| on_close.call(())>"Abbrechen"</button>
                        <button type="submit" class="primary-btn" disabled=is_submitting>
                            {move || if is_submitting.get() { "Speichert..." } else { "Speichern" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
