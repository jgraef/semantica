use leptos::{
    component,
    spawn_local,
    view,
    with,
    For,
    IntoView,
    Params,
    SignalGet,
    SignalUpdate,
};
use leptos_router::{
    use_navigate,
    use_params,
    Params,
};
use semantica_protocol::{
    auth::AuthSecret,
    user::UserId,
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
    utils::LogAndDiscardErrorExt,
};

fn login(user_id: UserId, auth_secret: AuthSecret) {
    log::debug!("login: {user_id}");

    spawn_local(
        async move {
            let Context { client, .. } = expect_context();
            let Storage {
                update_value: update_user_logins,
                ..
            } = use_user_logins();

            client.login(user_id, auth_secret).await?;

            update_user_logins.update(move |user_logins| {
                user_logins.logged_in = Some(user_id);
            });

            // in theory this is unecessary, since the register route redirects to / when
            // logged in. but somehow it doesn' re-render that part.
            use_navigate()("/", Default::default());

            Ok::<(), Error>(())
        }
        .log_and_discard_error(),
    );
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let Storage {
        value: user_logins, ..
    } = use_user_logins();

    view! {
        <div class="w-50 h-50 m-auto p-4">
            <h2 class="pb-4">"Welcome back!"</h2>

            <p class="pb-4">"Select one of your alters to get back into the game."</p>

            // todo: make this scrollable
            <div class="vstack gap-3">
                <For
                    each=move || {
                        let mut items = with!(|user_logins| {
                            user_logins.users
                                .values()
                                .cloned()
                                .collect::<Vec<_>>()
                        });
                        items.sort_by_cached_key(|user_login| user_login.name.to_lowercase());
                        items
                    }
                    key=|user_login| user_login.user_id
                    children=move |UserLogin { user_id, name, auth_secret, .. }| {
                        view!{
                            <button
                                type="button"
                                class="btn btn-secondary btn-lg"
                                on:click=move |_| {
                                    let auth_secret = auth_secret.clone();
                                    login(user_id, auth_secret);
                                }
                            >
                                {name}
                            </button>
                        }
                    }
                />
                <button
                    type="button"
                    class="btn btn-outline-secondary btn"
                    on:click=move |_| {
                        use_navigate()("/register", Default::default());
                    }
                >
                    "Create a new alter"
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn LoginWithUrl() -> impl IntoView {
    #[derive(Clone, Params, PartialEq)]
    struct LoginParams {
        user_id: UserId,
        auth_secret: AuthSecret,
    }

    let params = use_params::<LoginParams>();

    view! {
        {move || {
            match params.get() {
                Ok(params) => {
                    login(params.user_id, params.auth_secret);
                }
                Err(error) => {
                    log::error!("{error}");
                    todo!();
                }
            }
        }}
    }
}
