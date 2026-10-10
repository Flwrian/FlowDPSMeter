use std::{collections::HashMap, path::PathBuf, sync::{Arc,atomic::{AtomicU64,Ordering}}, io::Write};
use parking_lot::Mutex;
use serde::{Serialize,Deserialize};
use tauri::{Manager,State};
use crate::{party::Snapshot,pipeline::Pipeline,capture::{pcap_capturer::PcapCapturer,captured_payload::CapturedPayload}};

fn now()->i64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64 }
#[derive(Default,Clone,Serialize)]
#[serde(rename_all="camelCase")]
struct Diagnostics {
    mode:String, status:String, error:Option<String>, npcap:bool, administrator:bool,
    devices:Vec<String>, selected_device:Option<String>, connection:Option<String>, packets:u64,
    parsed:u64, duplicates:u64, losses:u64, recording:Option<String>, fixture:bool,
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
struct Settings {opacity:f64, always_on_top:bool, width:u32,height:u32,x:Option<i32>,y:Option<i32>, #[serde(default)] preferred_device:Option<String>,#[serde(default)] manual_party:Option<Vec<String>>,#[serde(default)] compact:bool,#[serde(default)] expanded_size:Option<(u32,u32)>}
impl Default for Settings {fn default()->Self{Self{opacity:0.96,always_on_top:true,width:620,height:600,x:None,y:None,preferred_device:None,manual_party:None,compact:false,expanded_size:None}}}
#[derive(Serialize,Deserialize,Default)]
#[serde(rename_all="camelCase")]
struct History {schema_version:u32, fights:Vec<Snapshot>}
struct Runtime {
    pipeline:Mutex<Pipeline>, diagnostics:Mutex<Diagnostics>, settings:Mutex<Settings>, history:Mutex<History>,
    capture:Mutex<Option<Arc<PcapCapturer>>>, generation:AtomicU64, recording:Mutex<Option<std::fs::File>>,
    directory:PathBuf, persistence_error:Mutex<Option<String>>,
    capture_stopped_at:AtomicU64,
}
impl Runtime {
    fn fresh_pipeline(&self)->Pipeline {
        let mut pipeline=Pipeline::new();pipeline.set_manual_party(self.settings.lock().manual_party.clone());pipeline
    }
    fn archive(&self, mut fights:Vec<Snapshot>) {
        if fights.is_empty(){return;}
        if self.persistence_error.lock().is_some(){return;}
        let mut history=self.history.lock();
        for mut fight in fights.drain(..) {
            // Replay and live fights can have identical timestamps; never overwrite a distinct record.
            if history.fights.iter().any(|s|s.id==fight.id) {fight.id=format!("{}-{}",fight.id,now());}
            history.fights.insert(0,fight);
        }
        history.fights.truncate(200);history.schema_version=1;
        if let Err(error)=crate::atomic_file::write(&self.directory.join("history.json"),&serde_json::to_vec(&*history).unwrap()) {
            *self.persistence_error.lock()=Some(format!("Historique non sauvegardé : {error}"));
        }
    }
    fn finish(&self, reason:&str) {
        let mut pipeline=self.pipeline.lock();
        let timestamp=if self.diagnostics.lock().mode=="live" {now()} else {pipeline.timestamp};
        pipeline.engine.finish(reason,timestamp);
        let mut fights=pipeline.take_finished();
        for fight in &mut fights {pipeline.label_skills(fight);}
        drop(pipeline);self.archive(fights);
    }
    fn stop(&self) {
        if self.diagnostics.lock().mode=="live" {self.capture_stopped_at.store(now() as u64,Ordering::SeqCst);}
        self.generation.fetch_add(1,Ordering::SeqCst);
        if let Some(capture)=self.capture.lock().take(){capture.stop();}
        self.finish("Capture arrêtée");
        self.recording.lock().take();
        let mut diag=self.diagnostics.lock();diag.mode="stopped".into();diag.status="Capture arrêtée".into();diag.recording=None;
    }
}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
struct Dashboard {snapshot:Snapshot,diagnostics:Diagnostics,settings:Settings,persistence_error:Option<String>,data_directory:String,
    roster_updates:u64,observed_players:Vec<String>,group_timeline:Vec<crate::party::Point>,party_members:Vec<crate::party::PartyMemberStatus>}

#[tauri::command]
fn dashboard(state:State<'_,Arc<Runtime>>,detail_member:Option<String>,overview:Option<bool>) -> Dashboard {
    let diag=state.diagnostics.lock().clone();
    let mut pipeline=state.pipeline.lock();
    if diag.mode=="live" {pipeline.engine.tick(now());}
    let timestamp=if diag.mode=="live" {now()} else {pipeline.timestamp};
    let (snapshot,group_timeline)=pipeline.dashboard_data(timestamp,detail_member.as_deref(),overview.unwrap_or(false));
    let mut fights=pipeline.take_finished();
    for fight in &mut fights {pipeline.label_skills(fight);}
    let mut diagnostics=diag;
    if diagnostics.mode=="live" {
        if let Some(error)=crate::capture::pcap_capturer::LAST_CAPTURE_ERROR.lock().unwrap().clone(){diagnostics.error=Some(error);}
    }
    diagnostics.packets=pipeline.packets;diagnostics.parsed=pipeline.parsed;
    diagnostics.duplicates=pipeline.duplicates();
    diagnostics.losses=pipeline.losses()+crate::capture::pcap_capturer::DROPPED_PACKETS.load(Ordering::Relaxed);
    let roster_updates=pipeline.roster_updates;
    let observed_players=pipeline.observed_players();
    let party_members=pipeline.engine.party_members();
    drop(pipeline);state.archive(fights);
    Dashboard {snapshot, diagnostics,settings:state.settings.lock().clone(),persistence_error:state.persistence_error.lock().clone(),
        data_directory:state.directory.display().to_string(),roster_updates,observed_players,group_timeline,party_members}
}

#[tauri::command]
fn capture_start(state:State<'_,Arc<Runtime>>,device:Option<String>)->Result<(),String>{
    let was_live=state.diagnostics.lock().mode=="live";
    let stopped_at=state.capture_stopped_at.load(Ordering::SeqCst) as i64;
    let can_resume=was_live || (state.diagnostics.lock().mode=="stopped" && stopped_at>0 && now()-stopped_at<30_000);
    state.stop();
    if !crate::platform::pcap::library_available(){return Err("Npcap est absent. Installe Npcap depuis npcap.com puis relance la capture.".into());}
    let devices=crate::capture::pcap_capturer::list_device_labels()?;
    if device.as_ref().is_some_and(|d| !devices.contains(d)) {return Err("Interface réseau indisponible.".into());}
    if can_resume {state.pipeline.lock().resume_capture();}
    else {let pipeline=state.fresh_pipeline();*state.pipeline.lock()=pipeline;}
    crate::capture::pcap_capturer::DROPPED_PACKETS.store(0,Ordering::Relaxed);
    let (tx,mut rx)=tokio::sync::mpsc::channel::<CapturedPayload>(8192);
    let capture=Arc::new(PcapCapturer::new(tx));capture.start();
    if !capture.is_running(){return Err("Impossible d’ouvrir la capture. Vérifie Npcap et les permissions administrateur.".into());}
    *state.capture.lock()=Some(capture);
    let generation=state.generation.fetch_add(1,Ordering::SeqCst)+1;
    {let mut diag=state.diagnostics.lock();diag.mode="live".into();diag.status="Recherche de la connexion Aion 2…".into();
        diag.error=None;diag.npcap=true;diag.devices=devices;diag.selected_device=device.clone();diag.connection=None;diag.fixture=false;}
    let runtime=state.inner().clone();
    tauri::async_runtime::spawn(async move {
        // Candidate flows are isolated until one proves it carries the game's records.
        type Key=(String,String,u16,u16,String);
        let mut candidates:HashMap<Key,crate::capture::flow_candidate::FlowCandidate>=HashMap::new();
        let mut locked:Option<Key>=None;
        let mut last_packet=now();
        while runtime.generation.load(Ordering::SeqCst)==generation {
            let packet=match tokio::time::timeout(std::time::Duration::from_secs(1),rx.recv()).await {
                Ok(Some(p))=>p,Ok(None)=>break,Err(_)=> {
                    if now()-last_packet>30_000 && locked.is_some() {
                        runtime.finish("Déconnexion");locked=None;candidates.clear();
                        let pipeline=runtime.fresh_pipeline();*runtime.pipeline.lock()=pipeline;
                        runtime.diagnostics.lock().connection=None;
                    }
                    continue;
                }
            };
            if runtime.generation.load(Ordering::SeqCst)!=generation {break;}
            if locked.is_some() && now()-last_packet>30_000 {
                runtime.finish("Déconnexion"); locked=None; candidates.clear();
                let pipeline=runtime.fresh_pipeline();*runtime.pipeline.lock()=pipeline; runtime.diagnostics.lock().connection=None;
            }
            if device.as_ref().is_some_and(|d|packet.device_name.as_ref()!=Some(d)){continue;}
            if crate::platform::window_detector::find_aion2_window_title().is_none(){
                if locked.take().is_some() {runtime.finish("Jeu fermé");let pipeline=runtime.fresh_pipeline();*runtime.pipeline.lock()=pipeline;candidates.clear();runtime.diagnostics.lock().connection=None;}
                continue;
            }
            let key=(packet.src_ip.clone().unwrap_or_default(),packet.dst_ip.clone().unwrap_or_default(),packet.src_port,packet.dst_port,
                packet.device_name.clone().unwrap_or_default());
            if locked.as_ref().is_some_and(|k|k!=&key){continue;}
            if locked.is_none() {
                // TLS is a different connection, not an Aion combat stream.
                if packet.data.len()>=3 && (0x14..=0x17).contains(&packet.data[0]) && packet.data[1]==3 {continue;}
                if candidates.len()>=64 && !candidates.contains_key(&key){
                    if let Some(old)=candidates.iter().min_by_key(|(_,c)|c.last_seen).map(|(k,_)|k.clone()){candidates.remove(&old);}
                }
                let entry=candidates.entry(key.clone()).or_default();
                if !entry.push(packet.clone()){continue;}
                let truncated=entry.truncated;
                let buffered=entry.take_packets();
                locked=Some(key.clone());last_packet=now();
                {let mut diag=runtime.diagnostics.lock();diag.connection=Some(format!("{} → {} · {}",packet.src_port,packet.dst_port,key.4));
                    diag.status="Connexion détectée · attente des identités et du combat".into();}
                let mut pipeline=runtime.pipeline.lock();
                pipeline.engine.transport_incomplete|=truncated;
                if runtime.generation.load(Ordering::SeqCst)!=generation {break;}
                for p in &buffered {
                    if let Some(file)=runtime.recording.lock().as_mut(){
                        if let Err(error)=writeln!(file,"{}",serde_json::to_string(&crate::replay::Record::Packet{payload:p.clone()}).unwrap()) {
                            runtime.diagnostics.lock().error=Some(format!("Enregistrement interrompu : {error}"));
                        }
                    }
                    pipeline.feed(p);
                }
                candidates.clear();
                continue;
            }
            last_packet=now();
            if let Some(file)=runtime.recording.lock().as_mut() {
                let record=crate::replay::Record::Packet{payload:packet.clone()};
                if let Err(error)=writeln!(file,"{}",serde_json::to_string(&record).unwrap()) {
                    runtime.diagnostics.lock().error=Some(format!("Enregistrement interrompu : {error}"));
                }
            }
            let mut pipeline=runtime.pipeline.lock();
            if crate::capture::pcap_capturer::DROPPED_PACKETS.load(Ordering::Relaxed)>0 {pipeline.engine.transport_incomplete=true;}
            if runtime.generation.load(Ordering::SeqCst)!=generation {break;}
            pipeline.feed(&packet);
            let active=pipeline.engine.is_active();
            let mut fights=pipeline.take_finished();for fight in &mut fights{pipeline.label_skills(fight);}
            drop(pipeline);runtime.archive(fights);
            runtime.diagnostics.lock().status=if active{"Combat du groupe en cours"}else{"Connecté · en attente d’un combat"}.into();
        }
    });Ok(())
}

#[tauri::command]
fn capture_stop(state:State<'_,Arc<Runtime>>){state.stop();}
#[tauri::command]
fn select_target(state:State<'_,Arc<Runtime>>,target:Option<i32>){state.pipeline.lock().engine.select_target(target);}
#[tauri::command]
fn new_combat(state:State<'_,Arc<Runtime>>){
    state.finish("Réinitialisation manuelle");
    state.pipeline.lock().reset_combat();
}

#[tauri::command]
fn party_save(state:State<'_,Arc<Runtime>>,names:Option<Vec<String>>)->Result<(),String>{
    let names=crate::pipeline::validate_manual_party(names)?;
    let mut pipeline=state.pipeline.lock();
    apply_party(&state,&mut pipeline,names)
}
fn apply_party(state:&Runtime,pipeline:&mut Pipeline,names:Option<Vec<String>>)->Result<(),String>{
    let mut current=state.settings.lock();
    let mut updated=current.clone();updated.manual_party=names.clone();
    crate::atomic_file::write(&state.directory.join("settings.json"),&serde_json::to_vec(&updated).unwrap()).map_err(|e|e.to_string())?;
    *current=updated;drop(current);
    let ts=if state.diagnostics.lock().mode=="live"{now()}else{pipeline.timestamp};
    pipeline.timestamp=ts;pipeline.set_manual_party(names.clone());
    if let Some(file)=state.recording.lock().as_mut(){
        writeln!(file,"{}",serde_json::to_string(&crate::replay::Record::PartySelection{timestamp:ts,names}).unwrap()).map_err(|e|e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
fn party_add(state:State<'_,Arc<Runtime>>,name:String)->Result<(),String>{
    let mut pipeline=state.pipeline.lock();
    let names=pipeline.party_with_added_member(&name)?;
    apply_party(&state,&mut pipeline,Some(names))
}
#[tauri::command]
fn party_remove(state:State<'_,Arc<Runtime>>,name:String)->Result<(),String>{
    let mut pipeline=state.pipeline.lock();
    let names=pipeline.party_without_member(&name)?;
    apply_party(&state,&mut pipeline,Some(names))
}
#[tauri::command]
async fn replay_text(state:State<'_,Arc<Runtime>>,text:String)->Result<(),String>{
    state.stop();
    let party=state.settings.lock().manual_party.clone();
    let (pipeline,fights,fixture)=tauri::async_runtime::spawn_blocking(move||crate::replay::replay_with_party(&text,party)).await.map_err(|e|e.to_string())??;
    *state.pipeline.lock()=pipeline;
    // Fixtures don't contaminate the player's real history.
    if !fixture{state.archive(fights);}
    let mut diag=state.diagnostics.lock();diag.mode="replay".into();diag.fixture=fixture;diag.connection=None;diag.error=None;
    diag.status=if fixture{"Replay · scénario synthétique de vérification"}else{"Replay · capture réseau locale"}.into();Ok(())
}
#[tauri::command]
fn recording_toggle(state:State<'_,Arc<Runtime>>)->Result<Option<String>,String>{
    let mut recording=state.recording.lock();
    if let Some(mut file)=recording.take(){file.flush().map_err(|e|e.to_string())?;file.sync_all().map_err(|e|e.to_string())?;
        state.diagnostics.lock().recording=None;return Ok(None);}
    if state.diagnostics.lock().mode!="live"{return Err("Démarre la capture avant d’enregistrer.".into());}
    let dir=state.directory.join("captures");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let path=dir.join(format!("capture-{}.apm.jsonl",now()));
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e|e.to_string())?;
    writeln!(file,"{}",serde_json::to_string(&crate::replay::Record::Header{version:1,fixture:false}).unwrap()).map_err(|e|e.to_string())?;
    if let Some(names)=state.settings.lock().manual_party.clone(){
        writeln!(file,"{}",serde_json::to_string(&crate::replay::Record::PartySelection{timestamp:now(),names:Some(names)}).unwrap()).map_err(|e|e.to_string())?;
    }
    let path=path.display().to_string();*recording=Some(file);state.diagnostics.lock().recording=Some(path.clone());Ok(Some(path))
}
#[tauri::command]
fn history_list(state:State<'_,Arc<Runtime>>)->Vec<serde_json::Value>{
    state.history.lock().fights.iter().map(|s|serde_json::json!({"id":s.id,"startedAt":s.started_at,"duration":s.duration,
        "damage":s.totals.damage,"healing":s.totals.healing,"members":s.members.len(),"reason":s.end_reason})).collect()
}
#[tauri::command]
fn history_load(state:State<'_,Arc<Runtime>>,id:String)->Result<Snapshot,String>{
    state.history.lock().fights.iter().find(|s|s.id==id).cloned().ok_or("Combat introuvable.".into())
}
#[tauri::command]
fn history_delete(state:State<'_,Arc<Runtime>>,id:String)->Result<(),String>{
    let mut history=state.history.lock();let mut updated=history.fights.clone();updated.retain(|s|s.id!=id);
    let replacement=History{schema_version:1,fights:updated};
    crate::atomic_file::write(&state.directory.join("history.json"),&serde_json::to_vec(&replacement).unwrap()).map_err(|e|e.to_string())?;
    *history=replacement;Ok(())
}
fn csv_label(value:&str)->String{
    let prefix=if value.starts_with(['=','+','-','@','\t','\r']){"'"}else{""};
    format!("\"{}{}\"",prefix,value.replace('"',"\"\""))
}
#[tauri::command]
fn export_combat(state:State<'_,Arc<Runtime>>,id:Option<String>,format:String)->Result<String,String>{
    let snap=if let Some(id)=id{history_load(state.clone(),id)?}else{
        let pipeline=state.pipeline.lock();pipeline.snapshot(if state.diagnostics.lock().mode=="live"{now()}else{pipeline.timestamp})};
    if snap.started_at.is_none(){return Err("Aucun combat à exporter.".into());}
    let content=match format.as_str(){
        "json"=>serde_json::to_vec_pretty(&snap).unwrap(),
        "csv"=> {
            let mut out="\u{feff}Nom,Classe,Présent,Durée,Dégâts,DPS,Contribution,Soins,HPS,Soins reçus,Dégâts reçus,Morts\r\n".to_string();
            for row in &snap.members {
                if !row.identity_confirmed {
                    out+=&format!("{},{},{},{:.3},,,,,,,,\r\n",csv_label(&row.member.name),csv_label(&row.member.class),row.member.present,snap.duration);
                    continue;
                }
                out+=&format!("{},{},{},{:.3},{},{:.3},{:.3},{},{:.3},{},{},{}\r\n",
                csv_label(&row.member.name),csv_label(&row.member.class),row.member.present,snap.duration,row.totals.damage,row.totals.dps,
                row.contribution,row.totals.healing,row.totals.hps,row.totals.healing_received,row.totals.damage_received,row.totals.deaths);}
            out.into_bytes()
        },_=>return Err("Format inconnu.".into())};
    let dir=state.directory.join("exports");std::fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let path=dir.join(format!("combat-{}.{}",now(),format));crate::atomic_file::write(&path,&content).map_err(|e|e.to_string())?;
    Ok(path.display().to_string())
}
#[tauri::command]
fn settings_save(state:State<'_,Arc<Runtime>>,window:tauri::WebviewWindow,settings:Settings)->Result<(),String>{
    if !(0.25..=1.0).contains(&settings.opacity){return Err("Opacité invalide.".into());}
    window.set_always_on_top(settings.always_on_top).map_err(|e|e.to_string())?;
    let mut current=state.settings.lock();
    current.opacity=settings.opacity;current.always_on_top=settings.always_on_top;current.preferred_device=settings.preferred_device;
    drop(current);
    crate::atomic_file::write(&state.directory.join("settings.json"),&serde_json::to_vec(&*state.settings.lock()).unwrap()).map_err(|e|e.to_string())
}
#[tauri::command]
fn compact_set(state:State<'_,Arc<Runtime>>,window:tauri::WebviewWindow,compact:bool)->Result<(),String>{
    let mut settings=state.settings.lock().clone();
    if settings.compact==compact {return Ok(());}
    let scale=window.scale_factor().map_err(|e|e.to_string())?;
    let current=window.inner_size().map_err(|e|e.to_string())?;
    let size=if compact {
        settings.expanded_size=Some(((current.width as f64/scale)as u32,(current.height as f64/scale)as u32));
        let members=state.pipeline.lock().engine.summary(now()).members.len()as u32;
        (400,(160+members*35).clamp(200,430))
    }else {settings.expanded_size.unwrap_or((620,600))};
    let min=if compact{(320,180)}else{(420,420)};
    window.set_min_size(Some(tauri::LogicalSize::new(min.0,min.1))).map_err(|e|e.to_string())?;
    window.set_size(tauri::LogicalSize::new(size.0,size.1)).map_err(|e|e.to_string())?;
    // Window callbacks may have updated geometry, and a quick party-add may
    // have updated membership while the native resize was in flight.
    let mut current=state.settings.lock();
    current.compact=compact;current.width=size.0;current.height=size.1;current.expanded_size=settings.expanded_size;
    crate::atomic_file::write(&state.directory.join("settings.json"),&serde_json::to_vec(&*current).unwrap()).map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
fn window_action(window:tauri::WebviewWindow,action:String)->Result<(),String>{
    match action.as_str(){"lock"=>window.set_ignore_cursor_events(true),"unlock"=>window.set_ignore_cursor_events(false),
        "minimize"=>window.minimize(),"close"=>window.close(),"recenter"=>{
            window.set_size(tauri::LogicalSize::new(620,600)).map_err(|e|e.to_string())?;window.center()
        },"overview"=>{
            window.set_size(tauri::LogicalSize::new(1120,780)).map_err(|e|e.to_string())?;window.center()
        },_=>return Err("Action inconnue.".into())}.map_err(|e|e.to_string())
}
#[derive(Serialize,Deserialize)]
struct Geometry {x:i32,y:i32,width:u32,height:u32,scale:f64}
#[tauri::command]
fn window_geometry(window:tauri::WebviewWindow)->Result<Geometry,String>{
    let pos=window.outer_position().map_err(|e|e.to_string())?;
    let size=window.inner_size().map_err(|e|e.to_string())?;
    Ok(Geometry{x:pos.x,y:pos.y,width:size.width,height:size.height,scale:window.scale_factor().map_err(|e|e.to_string())?})
}
#[tauri::command]
fn window_transform(window:tauri::WebviewWindow,geometry:Geometry,resize:bool)->Result<(),String>{
    if resize {
        let scale=window.scale_factor().map_err(|e|e.to_string())?;
        let compact=window.state::<Arc<Runtime>>().settings.lock().compact;
        let width=geometry.width.clamp((if compact{320.0}else{420.0}*scale)as u32,(3000.0*scale)as u32);
        let height=geometry.height.clamp((if compact{180.0}else{420.0}*scale)as u32,(2000.0*scale)as u32);
        window.set_size(tauri::PhysicalSize::new(width,height)).map_err(|e|e.to_string())?;
    }
    window.set_position(tauri::PhysicalPosition::new(geometry.x,geometry.y)).map_err(|e|e.to_string())
}
#[tauri::command]
fn open_data_folder(state:State<'_,Arc<Runtime>>)->Result<(),String>{
    std::process::Command::new("explorer.exe").arg(&state.directory).spawn().map_err(|e|e.to_string())?;Ok(())
}

pub fn run(){
    let _=tracing_subscriber::fmt().with_env_filter("warn").try_init();
    let app=tauri::Builder::default().setup(|app|{
        let directory=std::env::var_os("APM_DATA_DIR").map(PathBuf::from).unwrap_or(app.path().app_local_data_dir()?);
        std::fs::create_dir_all(&directory)?;
        let mut persistence_error=None;
        let history=match std::fs::read(directory.join("history.json")){
            Ok(bytes)=>match serde_json::from_slice::<History>(&bytes){Ok(h)if h.schema_version==1=>h,
                _=>{persistence_error=Some("Historique incompatible ou endommagé : le fichier est conservé sans écrasement.".into());History::default()}},
            Err(e)if e.kind()==std::io::ErrorKind::NotFound=>History{schema_version:1,fights:vec![]},
            Err(e)=>{persistence_error=Some(format!("Lecture historique impossible : {e}"));History::default()}};
        let mut settings=std::fs::read(directory.join("settings.json")).ok().and_then(|b|serde_json::from_slice::<Settings>(&b).ok()).unwrap_or_default();
        settings.manual_party=crate::pipeline::validate_manual_party(settings.manual_party).unwrap_or(None);
        let npcap=crate::platform::pcap::library_available();
        let devices=crate::capture::pcap_capturer::list_device_labels().unwrap_or_default();
        let mut pipeline=Pipeline::new();pipeline.set_manual_party(settings.manual_party.clone());
        let state=Arc::new(Runtime {pipeline:Mutex::new(pipeline),diagnostics:Mutex::new(Diagnostics{
            mode:"stopped".into(),status:if npcap{"Prêt · démarre la capture"}else{"Npcap absent · capture indisponible"}.into(),
            npcap,administrator:crate::platform::admin::is_admin(),devices,..Default::default()}),
            settings:Mutex::new(settings),history:Mutex::new(history),capture:Mutex::new(None),generation:AtomicU64::new(0),
            recording:Mutex::new(None),directory,persistence_error:Mutex::new(persistence_error),capture_stopped_at:AtomicU64::new(0)});
        if let Some(window)=app.get_webview_window("main"){
            let settings=state.settings.lock();
            window.set_always_on_top(settings.always_on_top)?;
            let min=if settings.compact{(320,180)}else{(420,420)};
            window.set_min_size(Some(tauri::LogicalSize::new(min.0,min.1)))?;
            window.set_size(tauri::LogicalSize::new(settings.width.clamp(min.0,2000),settings.height.clamp(min.1,1500)))?;
            if let (Some(x),Some(y))=(settings.x,settings.y){
                let visible=window.available_monitors()?.iter().any(|m|{
                    let p=m.position();let s=m.size();x>=p.x&&y>=p.y&&x<p.x+s.width as i32-100&&y<p.y+s.height as i32-60});
                if visible{window.set_position(tauri::PhysicalPosition::new(x,y))?;}
            }
        }
        app.manage(state.clone());
        let unlock=tauri::menu::MenuItem::with_id(app,"unlock","Afficher et déverrouiller",true,None::<&str>)?;
        let quit=tauri::menu::MenuItem::with_id(app,"quit","Quitter",true,None::<&str>)?;
        let menu=tauri::menu::Menu::with_items(app,&[&unlock,&quit])?;
        let mut tray=tauri::tray::TrayIconBuilder::new().tooltip("AionPartyMeter").menu(&menu).on_menu_event(|app,event|{
            if event.id.as_ref()=="unlock"{if let Some(w)=app.get_webview_window("main"){let _=w.set_ignore_cursor_events(false);let _=w.show();let _=w.unminimize();let _=w.set_focus();}}
            if event.id.as_ref()=="quit"{app.exit(0);}
        });
        if let Some(icon)=app.default_window_icon(){tray=tray.icon(icon.clone());}tray.build(app)?;
        Ok(())
    }).invoke_handler(tauri::generate_handler![dashboard,capture_start,capture_stop,select_target,new_combat,replay_text,
        recording_toggle,party_save,party_add,party_remove,compact_set,history_list,history_load,history_delete,export_combat,settings_save,window_action,window_geometry,window_transform,open_data_folder])
    .build(tauri::generate_context!()).expect("Impossible de lancer AionPartyMeter");
    app.run(|app,event|{
        if let tauri::RunEvent::WindowEvent{event,..}=&event {
            let state=app.state::<Arc<Runtime>>();
            let mut settings=state.settings.lock();
            match event {
                tauri::WindowEvent::Moved(pos)=>{settings.x=Some(pos.x);settings.y=Some(pos.y);},
                tauri::WindowEvent::Resized(size)=>{
                    let scale=app.get_webview_window("main").and_then(|w|w.scale_factor().ok()).unwrap_or(1.0);
                    if size.width>0&&size.height>0 {settings.width=(size.width as f64/scale)as u32;settings.height=(size.height as f64/scale)as u32;}
                },_=>{}
            }
        }
        if let tauri::RunEvent::Exit=event{
            let state=app.state::<Arc<Runtime>>();state.stop();
            if let Some(window)=app.get_webview_window("main"){
                let mut settings=state.settings.lock();
                if let Ok(size)=window.inner_size(){let scale=window.scale_factor().unwrap_or(1.0);settings.width=(size.width as f64/scale)as u32;settings.height=(size.height as f64/scale)as u32;}
                if let Ok(pos)=window.outer_position(){settings.x=Some(pos.x);settings.y=Some(pos.y);}
            }
            let _=crate::atomic_file::write(&state.directory.join("settings.json"),&serde_json::to_vec(&*state.settings.lock()).unwrap());
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csv_labels_quote_and_neutralize_formulas(){
        assert_eq!(csv_label("=SUM(1,2)"),"\"'=SUM(1,2)\"");
        assert_eq!(csv_label("Lumière \"Clerc\""),"\"Lumière \"\"Clerc\"\"\"");
    }
    #[test]
    fn old_settings_keep_party_and_geometry_with_compact_disabled(){
        let old=r#"{"opacity":0.8,"alwaysOnTop":true,"width":700,"height":600,"x":40,"y":80,"preferredDevice":null,"manualParty":["Viktor"]}"#;
        let mut settings:Settings=serde_json::from_str(old).unwrap();
        assert!(!settings.compact);assert_eq!(settings.manual_party,Some(vec!["Viktor".into()]));
        settings.compact=true;settings.expanded_size=Some((700,600));settings.width=420;settings.height=220;
        let restored:Settings=serde_json::from_slice(&serde_json::to_vec(&settings).unwrap()).unwrap();
        assert!(restored.compact);assert_eq!(restored.expanded_size,Some((700,600)));assert_eq!(restored.width,420);
    }
    #[test]
    fn history_is_atomic_bounded_and_reloadable(){
        let directory=std::env::temp_dir().join(format!("apm-history-test-{}-{}",std::process::id(),now()));
        std::fs::create_dir_all(&directory).unwrap();
        let runtime=Runtime{pipeline:Mutex::new(Pipeline::new()),diagnostics:Mutex::new(Diagnostics::default()),
            settings:Mutex::new(Settings::default()),history:Mutex::new(History::default()),capture:Mutex::new(None),
            generation:AtomicU64::new(0),recording:Mutex::new(None),directory:directory.clone(),persistence_error:Mutex::new(None),capture_stopped_at:AtomicU64::new(0)};
        let snapshot=runtime.pipeline.lock().snapshot(0);
        runtime.archive((0..205).map(|i|{let mut s=snapshot.clone();s.id=i.to_string();s}).collect());
        let saved:History=serde_json::from_slice(&std::fs::read(directory.join("history.json")).unwrap()).unwrap();
        assert_eq!(saved.schema_version,1);assert_eq!(saved.fights.len(),200);assert_eq!(saved.fights[0].id,"204");
        let bytes=std::fs::read(directory.join("history.json")).unwrap();
        *runtime.persistence_error.lock()=Some("Historique illisible".into());
        runtime.archive(vec![snapshot]);
        assert_eq!(bytes,std::fs::read(directory.join("history.json")).unwrap());
        std::fs::remove_file(directory.join("history.json")).unwrap();std::fs::remove_dir(directory).unwrap();
    }
}
