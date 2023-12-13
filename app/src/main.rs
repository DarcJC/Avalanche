use avalanche_engine::core::instance::EngineInstance;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    let mut instance = EngineInstance::default();
    instance.run();

    Ok(())
}
