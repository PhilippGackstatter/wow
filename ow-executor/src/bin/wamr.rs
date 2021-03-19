fn main() -> anyhow::Result<()> {
    let wasm_module_bytes = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();
    #[cfg(feature = "wamr_rt")]
    ow_wamr::run_module(wasm_module_bytes)
}
