use leptos::{
    component,
    create_node_ref,
    create_rw_signal,
    event_target_value,
    html::Input,
    spawn_local,
    update,
    view,
    IntoView,
    SignalGet,
    SignalSet,
};
use leptos_use::use_debounce_fn_with_arg;
use strum::{
    EnumIs,
    EnumMessage,
};

use super::{
    expect_context,
    Context,
};
use crate::{
    error::Error,
    storage::{
        use_user_logins,
        Storage,
        UserLogin,
    },
    utils::spawn_local_and_handle_error,
};

#[component]
pub fn RegisterPage() -> impl IntoView {
    #[derive(Clone, Copy, Debug, EnumMessage, EnumIs)]
    enum NameState {
        #[strum(message = "Enter your name")]
        Empty,
        #[strum(message = "Looks good!")]
        Valid,
    }

    let name_input_state = create_rw_signal(NameState::Empty);
    let name_input_field = create_node_ref::<Input>();

    view! {
        <div class="w-50 h-50 m-auto p-4">
            <h2 class="pb-4">"New here?"</h2>

            <p class="pb-4">"Welcome to Semantica! Enter your name and we're ready to go."</p>

            <form
                class="form-floating"
                on:submit=move |event| {
                    event.prevent_default();

                    let name = name_input_field.get().unwrap().value();
                    if !name.is_empty() {
                        log::debug!("submit register. name={name:?}");

                        spawn_local_and_handle_error(async move {
                            let Context { client, .. } = expect_context();
                            let response = client.register(name.clone()).await?;

                            // no need to authenticate the client, as the register endpoint does that too.

                            let user_login = UserLogin {
                                user_id: response.user_id,
                                name,
                                auth_secret: response.auth_secret,
                                login_link_noticed: false,
                            };
                            log::debug!("{user_login:?}");

                            let Storage { update_value: user_logins, .. } = use_user_logins();
                            update!(|user_logins| {
                                user_logins.logged_in = Some(user_login.user_id.clone());
                                user_logins.users.insert(user_login.user_id.clone(), user_login);
                            });

                            Ok::<(), Error>(())
                        });

                    }
                }
            >
                <div class="form-floating mb-3">
                    <input
                        type="text"
                        class="form-control form-control-lg"
                        class:is-valid=move || name_input_state.get().is_valid()
                        id="register_name"
                        required
                        node_ref=name_input_field
                        on:input=move |event| {
                            use_debounce_fn_with_arg(
                                move |event| {
                                    let value = event_target_value(&event);
                                    name_input_state.set(if value.is_empty() { NameState::Empty } else { NameState::Valid });
                                },
                                200.0
                            )(event);
                        }
                    />
                    <label for="register_name">
                        {move || name_input_state.get().get_message()}
                    </label>
                </div>
                //<div class="d-flex flex-row">
                //    <button type="button" class="btn btn-primary w-25 ms-auto">"Let's go"</button>
                //</div>
            </form>
        </div>
    }
}
