use serde::{Deserialize, Serialize};
use crate::{pipeline::Pipeline, party::{CombatEvent, Identity, Member, Snapshot}, capture::captured_payload::CapturedPayload};

/// Real JSONL files contain network `packet` records and explicit user party
/// selections. `identity`, `roster` and `event` records are test fixtures only.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type", rename_all="camelCase")]
pub enum Record {
    Header { version:u32, fixture:bool },
    Packet { payload:CapturedPayload },
    /// Explicit user configuration, not a decoded network roster.
    PartySelection {timestamp:i64,names:Option<Vec<String>>},
    Identity { timestamp:i64, names:std::collections::HashMap<String,String>, local_id:i32,
        summons:std::collections::HashMap<String,i32>, mobs:Vec<i32> },
    Roster { timestamp:i64, members:Vec<Member>, complete:bool },
    Event { event:CombatEvent },
    Boundary { timestamp:i64, reason:String },
}

pub fn replay(text:&str) -> Result<(Pipeline,Vec<Snapshot>,bool),String> {
    replay_with_party(text,None)
}

/// A legacy capture may have no recorded selection. Start it with the user's
/// explicit party, unless the file is a synthetic fixture. Recorded selections
/// take precedence at their original timestamps.
pub fn replay_with_party(text:&str,initial_party:Option<Vec<String>>) -> Result<(Pipeline,Vec<Snapshot>,bool),String> {
    if text.len() > 64*1024*1024 { return Err("Fichier trop volumineux (64 Mo maximum).".into()); }
    let mut pipeline=Pipeline::new();
    let mut fixture=false;
    let mut header=false;
    let mut finished=vec![];
    for (line_number,line) in text.lines().enumerate() {
        if line.trim().is_empty() {continue;}
        let record:Record=serde_json::from_str(line).map_err(|e|format!("Ligne {} : {e}",line_number+1))?;
        if !header {
            match record { Record::Header {version:1,fixture:f} => {
                fixture=f;header=true;
                if !fixture {pipeline.set_manual_party(crate::pipeline::validate_manual_party(initial_party.clone())?);}
                continue;
            },
                _ => return Err("En-tête AionPartyMeter v1 absent ou non compatible.".into()) }
        }
        match record {
            Record::Header {..}=>return Err("En-tête répété.".into()),
            Record::Packet {payload}=>pipeline.feed(&payload),
            Record::PartySelection {timestamp,names}=>{
                pipeline.timestamp=timestamp;pipeline.set_manual_party(crate::pipeline::validate_manual_party(names)?);
            },
            Record::Identity {timestamp,names,local_id,summons,mobs} if fixture => {
                pipeline.timestamp=timestamp;
                let names:std::collections::HashMap<i32,String>=names.into_iter().map(|(k,v)|k.parse::<i32>().map(|k|(k,v))).collect::<Result<_,_>>().map_err(|_|"Identité invalide.".to_string())?;
                let summons:std::collections::HashMap<i32,i32>=summons.into_iter().map(|(k,v)|k.parse::<i32>().map(|k|(k,v))).collect::<Result<_,_>>().map_err(|_|"Invocation invalide.".to_string())?;
                let local_name=names.get(&local_id).cloned();
                let summon_ids=summons.keys().copied().collect();
                pipeline.engine.sync_identity(Identity {names,local_id:Some(local_id),local_name,summons,summon_ids,
                    mobs:mobs.into_iter().collect(),..Default::default()});
            },
            Record::Roster {timestamp,members,complete} if fixture=> {
                pipeline.timestamp=timestamp;pipeline.engine.roster(members,complete,timestamp);
            },
            Record::Event {event} if fixture=> {
                pipeline.timestamp=event.timestamp;
                if event.kind==crate::party::EventKind::Death {pipeline.engine.death(event.target,event.timestamp);}
                else {pipeline.engine.ingest(event);}
            },
            Record::Boundary {timestamp,reason} if fixture=> {
                pipeline.timestamp=timestamp;pipeline.engine.boundary(&reason,timestamp);
            },
            _=>return Err("Événements synthétiques interdits dans une capture réelle.".into()),
        }
        for mut snap in pipeline.take_finished() { pipeline.label_skills(&mut snap);finished.push(snap); }
    }
    if !header {return Err("Fichier vide.".into());}
    pipeline.engine.finish("Fin du replay",pipeline.timestamp);
    for mut snap in pipeline.take_finished() {pipeline.label_skills(&mut snap);finished.push(snap);}
    Ok((pipeline,finished,fixture))
}
