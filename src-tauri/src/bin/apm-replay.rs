fn main() {
    let args:Vec<_>=std::env::args().skip(1).collect();
    let path=args.first().expect("apm-replay <capture.apm.jsonl> [snapshot.json] [--party=pseudo1,pseudo2] [--all-fights] [--diagnostic]");
    let text=std::fs::read_to_string(path).expect("Lecture capture");
    let party=args.iter().find_map(|a|a.strip_prefix("--party=")).map(|s|s.split(',').map(String::from).collect());
    match a2tools_dps_meter_lib::replay::replay_with_party(&text,party) {
        Ok((pipeline,fights,fixture))=> {
            if args.iter().any(|a|a=="--diagnostic") {
                eprintln!("{}",serde_json::json!({"identities":pipeline.storage.authoritative_names(),
                    "roster":pipeline.storage.get_party_members().keys().collect::<Vec<_>>(),
                    "rosterUpdates":pipeline.roster_updates,"packets":pipeline.packets,"losses":pipeline.losses(),"fights":fights.len()}));
            }
            let mut snap=pipeline.snapshot(pipeline.timestamp);
            if fixture {snap.warnings.push("Scénario synthétique de vérification ; ce rapport ne provient pas du jeu.".into());}
            let json=if args.iter().any(|a|a=="--all-fights"){serde_json::to_string_pretty(&fights).unwrap()}else{serde_json::to_string_pretty(&snap).unwrap()};
            if let Some(path)=args.iter().skip(1).find(|a|!a.starts_with("--")) {std::fs::write(path,json).unwrap();}
            else {println!("{json}");}
            eprintln!("Replay terminé ; données synthétiques : {fixture}");
        },
        Err(error)=> {eprintln!("{error}");std::process::exit(1);}
    }
}
