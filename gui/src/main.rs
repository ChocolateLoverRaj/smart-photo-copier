use floem::{
    action::open_file,
    file::FileDialogOptions,
    prelude::{create_rw_signal, SignalGet, SignalUpdate},
    views::{button, dyn_view, v_stack, Decorators},
    IntoView,
};
use paths_view::paths_view;

mod paths_view;

#[tokio::main]
async fn main() {
    floem::launch(|| {
        let in_progress = create_rw_signal(false);
        let sources = create_rw_signal(Default::default());
        let destination = create_rw_signal(Default::default());
        let check_folders = create_rw_signal(Default::default());
        v_stack((
            button("Select source folder")
                .disabled(move || in_progress.get())
                .on_click_cont(move |_| {
                    open_file(
                        FileDialogOptions::new()
                            .select_directories()
                            .multi_selection()
                            .title("Select source folder"),
                        move |file_info| {
                            if let Some(file_info) = file_info {
                                sources.set(file_info.path);
                            }
                        },
                    );
                }),
            paths_view(sources),
            button("Select destination folder")
                .disabled(move || in_progress.get())
                .on_click_cont(move |_| {
                    open_file(
                        FileDialogOptions::new()
                            .select_directories()
                            .title("Select destination folder"),
                        move |file_info| {
                            if let Some(file_info) = file_info.and_then(|file_info| {
                                file_info.path.first().map(|path_buf| path_buf.to_owned())
                            }) {
                                destination.set(Some(file_info));
                            }
                        },
                    );
                }),
            dyn_view(move || match destination.get() {
                Some(destination) => destination.to_string_lossy().into_any(),
                None => "<not selected>".into_any(),
            }),
            button("Select folders to check")
                .disabled(move || in_progress.get())
                .on_click_cont(move |_| {
                    open_file(
                        FileDialogOptions::new()
                            .select_directories()
                            .multi_selection()
                            .title("Select Folders to check"),
                        move |file_info| {
                            if let Some(file_info) = file_info {
                                check_folders.set(file_info.path);
                            }
                        },
                    );
                }),
            paths_view(check_folders),
            button("Start")
                .disabled(move || in_progress.get())
                .on_click_cont(move |_| {
                    in_progress.set(true);
                }),
        ))
    });
}
