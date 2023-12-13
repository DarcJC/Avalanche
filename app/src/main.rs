use avalanche_window::get_window_manager_mut;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let _ = get_window_manager_mut().await.create_main_window();

    loop {
        get_window_manager_mut().await.handle_events();
    }

    Ok(())
}
