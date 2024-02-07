pub mod index;

use leptos::{
    component,
    view,
    DynAttrs,
    IntoView,
    Oco,
    Signal,
    SignalGet,
    SignalSet,
};
use leptos_meta::{
    provide_meta_context,
    Html,
};
use leptos_router::{
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

use self::index::Index;
use crate::storage::{
    use_storage,
    Storage,
    StorageKey,
    UserLogins,
};

const GITHUB_PAGE: &'static str = "https://github.com/jgraef/semantica";

#[derive(Clone, Debug)]
pub struct Context {
    user_logins: Storage<UserLogins>,
}

pub fn provide_context() -> Context {
    let context = Context {
        user_logins: use_storage::<UserLogins>(StorageKey::UserLogins),
    };

    leptos::provide_context(context.clone());

    context
}

pub fn expect_context() -> Context {
    leptos::expect_context()
}

#[component]
pub fn BootstrapIcon(#[prop(into)] icon: Oco<'static, str>) -> impl IntoView {
    view! { <i class={format!("bi bi-{icon}")}></i> }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context();

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
                            <ul class="navbar-nav me-auto mb-2 mb-lg-0">
                                <li class="nav-item">
                                    <a class="nav-link active" aria-current="page" href="#">"Home"</a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link" href="#">"Link"</a>
                                </li>
                            </ul>
                        </div>
                    </div>
                </nav>

                <main class="main d-flex flex-column w-100 h-100 mw-100 mh-100">
                    <Routes>
                        <Route path="/" view=Index />
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
