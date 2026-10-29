use jose::{Jose, PlayerIdentity, regression_closer_role};
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("version") => println!(
            "jose {} schema={} policy={}",
            env!("CARGO_PKG_VERSION"),
            jose::SCHEMA_VERSION,
            jose::POLICY_VERSION
        ),
        Some("mock-demo") => {
            let mut system = Jose::default();
            let identity = PlayerIdentity {
                provider: "mock".into(),
                model: "regression-specialist".into(),
                endpoint: "local://mock".into(),
                quantisation: None,
                context_configuration: "curated-v1".into(),
                tool_configuration: "bounded-write".into(),
                reasoning_configuration: "standard".into(),
                organisation_policy: "local".into(),
            };
            let id = system.register_player(identity);
            system.add_role(regression_closer_role());
            println!(
                "Registered mock player {id:?}; role regression_closer/v1 available. No task execution performed."
            );
        }
        Some("benchmark-demo") => {
            println!(
                "Phase II benchmark inputs are externally observed outcomes in fixtures/benchmark_pairs.json."
            );
            println!(
                "Load them with the jose library, then call Jose::benchmark_summary; Jose does not execute benchmark tasks."
            );
        }
        _ => {
            eprintln!("Usage: jose version | jose mock-demo | jose benchmark-demo");
            std::process::exit(2);
        }
    }
}
