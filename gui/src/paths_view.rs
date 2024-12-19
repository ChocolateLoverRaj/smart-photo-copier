use std::path::PathBuf;

use floem::{
    prelude::SignalGet,
    views::{dyn_view, list},
    IntoView,
};

pub fn paths_view<T: SignalGet<Vec<PathBuf>> + 'static>(paths: T) -> impl IntoView {
    dyn_view(move || match paths.get().len() {
        0 => "<none selected>".into_any(),
        _ => list(
            paths
                .get()
                .iter()
                .map(|path| path.to_string_lossy().into_view()),
        )
        .into_any(),
    })
}
