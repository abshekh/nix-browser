//! [leptos_query] helpers for working with [server] fns, and useful widgets.
use cfg_if::cfg_if;
use leptos::*;
use leptos_query::*;
use std::{fmt::Display, future::Future, hash::Hash, str::FromStr};
use tracing::instrument;

/// The result type of Leptos [server] function returning a `T`
pub type ServerFnResult<T> = Result<T, ServerFnError>;

/// Sensible [QueryOptions] defaults for an app
pub fn query_options<V>() -> QueryOptions<V> {
    QueryOptions {
        // Disable staleness so the query is not refetched on every route switch.
        stale_time: None,
        ..Default::default()
    }
}

/// Like [use_query] but specifically meant for server functions, does logging
/// via [tracing] and uses [query_options] always.
///
/// In order to be able to log the name of the server fns, we unfortunately must
/// require them to be 1-ary functions taking tuples, due to a limitation with
/// Rust type system around Fn trait.
///
/// Arguments
/// * `k`: The argument to the server fn
/// * `fetcher`: The server fn to call
#[instrument(
    name = "use_server_query",
    skip(cx, k, fetcher),
    fields(
        fetcher = std::any::type_name::<F>(),
        render_mode=LEPTOS_MODE
    )
)]
pub fn use_server_query<K, V, F, Fu>(
    cx: Scope,
    k: impl Fn() -> K + 'static,
    fetcher: F,
) -> QueryResult<ServerFnResult<V>, impl RefetchFn>
where
    K: Hash + Eq + Clone + std::fmt::Debug + 'static,
    ServerFnResult<V>: Clone + Serializable + 'static,
    Fu: Future<Output = ServerFnResult<V>> + 'static,
    F: Fn(K) -> Fu + 'static,
{
    let span = tracing::Span::current();
    tracing::debug!("use_query");
    leptos_query::use_query(
        cx,
        k,
        move |k| {
            let _enter = span.enter();
            tracing::info!("calling server fn");
            fetcher(k)
        },
        query_options::<ServerFnResult<V>>(),
    )
}

const LEPTOS_MODE: &str = {
    cfg_if! { if #[cfg(feature="ssr")] {
        "ssr"
    } else if #[cfg(feature="hydrate")] {
        "hydrate"
    } else {
        compile_error!("Either ssr or hydrate feature must be enabled");
    }}
};

/// Input element component to pass arguments to a [leptos_query] query
///
/// A label, input element, and datalist are rendered, as well as error div.
/// [FromStr::from_str] is used to parse the input value into `K`.
///
/// Arguments:
/// * `id`: The id of the input element
/// * `suggestions`: The initial suggestions to show in the datalist
/// * `query`: Input element value is initialized with this [ReadSignal]
/// * `set_query`: Input element will set this [WriteSignal]
#[component]
pub fn QueryInput<K>(
    cx: Scope,
    id: &'static str,
    /// Initial suggestions to show in the datalist
    suggestions: Vec<K>,
    query: ReadSignal<K>,
    set_query: WriteSignal<K>,
) -> impl IntoView
where
    K: ToString + FromStr + Hash + Eq + Clone + Display + 'static,
    <K as std::str::FromStr>::Err: Display,
{
    let datalist_id = &format!("{}-datalist", id);
    // Input query to the server fn
    // Errors in input element (based on [FromStr::from_str])
    let (input_err, set_input_err) = create_signal(cx, None::<String>);
    view! { cx,
        <label for=id>"Load a Nix flake"</label>
        <input
            list=datalist_id
            id=id.to_string()
            type="text"
            class="w-full p-1 font-mono"
            on:change=move |ev| {
                match FromStr::from_str(&event_target_value(&ev)) {
                    Ok(url) => {
                        set_query(url);
                        set_input_err(None)
                    }
                    Err(e) => set_input_err(Some(e.to_string())),
                }
            }

            prop:value=move || query().to_string()
        />
        <span class="text-red-500">{input_err}</span>
        // TODO: use local storage, and cache user's inputs
        <datalist id=datalist_id>
            {suggestions
                .iter()
                .map(|s| view! { cx, <option value=s.to_string()></option> })
                .collect_view(cx)}

        </datalist>
    }
}

/// Button component to refresh the given [leptos_query] query.
///
/// Arguments
/// * `result`: The query result to refresh
/// * `query`: The value to pass to [invalidate_query]
#[component]
pub fn RefetchQueryButton<K, V, R, F>(
    cx: Scope,
    result: QueryResult<ServerFnResult<V>, R>,
    query: F,
) -> impl IntoView
where
    K: Hash + Eq + Clone + std::fmt::Debug + 'static,
    ServerFnResult<V>: Clone + Serializable + 'static,
    R: RefetchFn,
    F: Fn() -> K + 'static,
{
    view! { cx,
        <button
            class="p-1 text-white shadow border-1 bg-primary-700 disabled:bg-base-400 disabled:text-black"
            disabled=move || result.is_fetching.get()
            on:click=move |_| {
                let k = query();
                tracing::debug!("Invalidating query");
                use_query_client(cx).invalidate_query::<K, ServerFnResult<V>>(k);
            }
        >

            {move || if result.is_fetching.get() { "Fetching..." } else { "Re-fetch" }}
        </button>
    }
}
