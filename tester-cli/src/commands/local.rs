pub fn print_start_node_hint() {
    println!("Start a fast local tester node:");
    println!(
        "cargo run -p gaia-node --features gaia-runtime/fast-local -- --dev --tmp --rpc-external --unsafe-rpc-external"
    );
    println!("This preset funds seeded personas (Alice..Ferdie) for immediate local testing.");
}

pub fn print_reset_hint() {
    println!("Reset local chain data by removing the previous base path or using --tmp.");
}

pub fn print_metadata_hint() {
    println!("Refresh committed metadata artifact:");
    println!("1) Start the node locally (WS: ws://127.0.0.1:9944)");
    println!(
        "2) cargo run -p gaia-tester-cli --bin refresh_metadata -- ws://127.0.0.1:9944 tester-cli/artifacts/gaia.scale"
    );
    println!("3) Rebuild the tester CLI against the refreshed artifact");
}
