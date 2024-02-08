pub mod index;
pub mod login;
pub mod register;

use leptos::{
    component,
    spawn_local,
    view,
    with,
    DynAttrs,
    IntoView,
    Oco,
    Signal,
    SignalGet,
    SignalSet,
    SignalUpdate,
    WriteSignal,
};
use leptos_meta::{
    provide_meta_context,
    Html,
};
use leptos_router::{
    Redirect,
    Route,
    Router,
    Routes,
    A,
};
use leptos_use::{
    use_color_mode,
    ColorMode,
    UseColorModeReturn,
};
use semantica_client::Client;
use url::Url;

use self::{
    index::IndexPage,
    login::{
        LoginPage,
        LoginWithUrl,
    },
    register::RegisterPage,
};
use crate::{
    error::Error,
    storage::{
        use_user_logins,
        Storage,
        UserLogin,
        UserLogins,
    },
    utils::LogAndDiscardErrorExt,
};

const GITHUB_PAGE: &'static str = "https://github.com/jgraef/semantica";

#[component]
pub fn BootstrapIcon(#[prop(into)] icon: Oco<'static, str>) -> impl IntoView {
    view! { <i class={format!("bi bi-{icon}")}></i> }
}

fn get_api_url() -> Option<Url> {
    let mut url: Url = gloo_utils::document().base_uri().ok()??.parse().ok()?;
    log::debug!("base_url: {url}");

    {
        let mut path_segments = url.path_segments_mut().ok()?;
        path_segments.push("api");
        path_segments.push("v1");
    }

    log::debug!("api_url: {url}");

    Some(url)
}

#[derive(Clone)]
pub struct Context {
    client: Client,
    user_logins: Signal<UserLogins>,
    update_user_logins: WriteSignal<UserLogins>,
    no_accounts_yet: Signal<bool>,
    logged_in_user: Signal<Option<UserLogin>>,
    is_logged_in: Signal<bool>,
}

fn provide_context() -> Context {
    let client = Client::new(get_api_url().expect("could not get api url"));

    let Storage {
        value: user_logins,
        update_value: update_user_logins,
        ..
    } = use_user_logins();

    let no_accounts_yet =
        Signal::derive(move || with!(|user_logins| { user_logins.users.is_empty() }));

    let logged_in_user = Signal::derive(move || {
        with!(|user_logins| {
            user_logins
                .logged_in
                .as_ref()
                .and_then(|user_id| user_logins.users.get(user_id))
                .cloned()
        })
    });

    let is_logged_in = Signal::derive(move || with!(|user_logins| user_logins.logged_in.is_some()));

    let context = Context {
        client,
        user_logins,
        update_user_logins,
        no_accounts_yet,
        logged_in_user,
        is_logged_in,
    };

    leptos::provide_context(context.clone());

    context
}

pub fn expect_context() -> Context {
    leptos::expect_context()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let Context {
        client,
        is_logged_in,
        update_user_logins,
        ..
    } = provide_context();

    let (bs_theme, toggle_theme, theme_icon) = {
        let UseColorModeReturn { mode, set_mode, .. } = use_color_mode();
        let bs_theme = Signal::derive(move || {
            match mode.get() {
                ColorMode::Dark => "dark",
                _ => "light",
            }
        });
        let toggle_theme = move || {
            let current = mode.get();
            let new = match current {
                ColorMode::Dark => ColorMode::Light,
                _ => ColorMode::Dark,
            };
            set_mode.set(new);
        };
        let theme_icon = Signal::derive(move || {
            match mode.get() {
                ColorMode::Dark => "moon-stars-fill",
                _ => "sun-fill",
            }
        });
        (bs_theme, toggle_theme, theme_icon)
    };

    view! {
        <Html
            attr:data-bs-theme=bs_theme
        />
        <Router>
            <div class="d-flex flex-column" style="height: 100vh; width: 100vw">
                <nav class="navbar navbar-expand-lg bg-body-tertiary">
                    <div class="container-fluid">
                        <A class="navbar-brand" href="/">"Semantica"</A>
                        <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbar_content" aria-controls="navbar_content" aria-expanded="false" aria-label="Toggle navigation">
                            <span class="navbar-toggler-icon"></span>
                        </button>
                        <div class="collapse navbar-collapse" id="navbar_content">
                            <div class="navbar-nav me-auto">

                            </div>
                            <div class="navbar-nav">
                                <button
                                    type="button"
                                    class="nav-link btn btn-link"
                                    on:click=move |_| toggle_theme()
                                >
                                    {move || view! { <BootstrapIcon icon={theme_icon.get()} /> }}
                                </button>
                                {move || {
                                    let client = client.clone();
                                    is_logged_in.get().then(move || {
                                        view!{
                                            <button
                                                type="button"
                                                class="nav-link btn btn-link"
                                                on:click=move |_| {
                                                    let client = client.clone();
                                                    spawn_local(async move {
                                                        client.logout().await?;
                                                        Ok::<(), Error>(())
                                                    }.log_and_discard_error());

                                                    update_user_logins.update(|user_logins| user_logins.logged_in = None);
                                                }
                                            >
                                                <BootstrapIcon icon="door-open-fill" />
                                            </button>
                                        }
                                    })
                                }}
                            </div>
                        </div>
                    </div>
                </nav>

                <main class="main d-flex flex-column w-100 h-100 mw-100 mh-100">
                    <Routes>
                        <Route path="/" view=IndexPage />
                        <Route path="/register" view=move || {
                            if is_logged_in.get() {
                                view!{ <Redirect path="/" /> }.into_view()
                            }
                            else {
                                view!{ <RegisterPage /> }.into_view()
                            }
                        } />
                        <Route path="/login" view=move || {
                            if is_logged_in.get() {
                                view!{ <Redirect path="/" /> }.into_view()
                            }
                            else {
                                view!{ <LoginPage /> }.into_view()
                            }
                        } />
                        <Route path="/login/:user_id/:auth_secret" view=LoginWithUrl />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div class="h-100 w-100 pt-3 px-4">
            <h1>"404 - Not found"</h1>
        </div>
    }
}
