//! Test that validation catches inconsistent profile switching buttons

use joy2_rs::mapping::config::Config;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    println!("Testing validation with invalid config (missing CycleProfiles)...\n");
    
    match Config::load("configs/test_invalid.toml") {
        Ok(_) => {
            eprintln!("❌ UNEXPECTED: Config should have failed validation!");
            std::process::exit(1);
        }
        Err(e) => {
            println!("✅ Validation correctly caught the error:");
            println!("   {}\n", e);
        }
    }
}
