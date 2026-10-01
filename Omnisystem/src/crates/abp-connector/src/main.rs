//! `abp-omni`: AgenticBotPlatform from Omnisystem's command line.
//!
//!     abp-omni status | bots | models | modules | swarms | localai | lab
//!     abp-omni ask <from-bot> <to-bot> <prompt…>      abp-omni chat <model|auto> <prompt…>
//!     abp-omni call <module> <operation> [json]       abp-omni swarm <id> <goal…>
//!     abp-omni advice transfer <src> <dst> | llm <model> | memory | stability
//!     abp-omni raw <GET|POST> <path> [json]
//!
//! ABP_URL (default http://127.0.0.1:8787) and ABP_KEY (an integration key, "omnisystem" preset).

use abp_connector::{AbpClient, COMMANDS};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        eprintln!("usage: abp-omni <{}> [args]\nABP_URL and ABP_KEY say which ABP and with which key.", COMMANDS.join("|"));
        std::process::exit(2);
    };
    let client = AbpClient::from_env();
    match client.run(cmd, &args[1..]).await {
        Ok(v) => {
            if let Some(reply) = v.get("reply").and_then(|r| r.as_str()) {
                println!("{reply}");
            } else {
                println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
            }
        }
        Err(e) => {
            eprintln!("abp-omni: {e}");
            std::process::exit(1);
        }
    }
}
