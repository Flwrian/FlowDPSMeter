//! A single connection's live/replay pipeline. TCP sequence accounting happens
//! before framing; packet payload equality is never used for deduplication.
use std::collections::HashMap;
use std::sync::Arc;
use crate::capture::{captured_payload::CapturedPayload, stream_assembler::StreamAssembler, stream_processor::StreamProcessor};
use crate::combat::data_storage::DataStorage;
use crate::i18n::lookup::{SkillLookup, NpcLookup};
use crate::party::{PartyEngine, Identity, Signal, EventKind, Member, class_label};

#[derive(Default)]
pub struct TcpOrder {
    next: Option<u32>,
    pending: Vec<(u32, Vec<u8>)>,
    gap_since: Option<i64>,
    pub losses: u64,
    pub duplicates: u64,
}

impl TcpOrder {
    pub fn push(&mut self, seq: u32, data: &[u8], timestamp: i64) -> Vec<Vec<u8>> {
        if data.is_empty() { return vec![]; }
        let next = *self.next.get_or_insert(seq);
        let delta = seq.wrapping_sub(next) as i32;
        if delta < 0 && (-i64::from(delta)) >= data.len() as i64 {
            self.duplicates += 1; return vec![];
        }
        if self.pending.iter().any(|(s,d)| *s == seq && d == data) {
            self.duplicates += 1; return vec![];
        }
        self.pending.push((seq,data.to_vec()));
        let mut out = vec![];
        loop {
            let next = self.next.unwrap();
            let pos = self.pending.iter().position(|(s,d)| {
                let diff = s.wrapping_sub(next) as i32;
                diff <= 0 && (-(diff as i64)) < d.len() as i64
            });
            if let Some(pos) = pos {
                let (start,mut data) = self.pending.remove(pos);
                let trim = next.wrapping_sub(start) as usize;
                if trim > 0 { data.drain(..trim); }
                self.next = Some(next.wrapping_add(data.len() as u32));
                out.push(data);
                self.gap_since = None;
                self.pending.retain(|(s,d)| {
                    let diff = s.wrapping_sub(self.next.unwrap()) as i32;
                    diff > 0 || (-(diff as i64)) < d.len() as i64
                });
                continue;
            }
            if !self.pending.is_empty() {
                let since = *self.gap_since.get_or_insert(timestamp);
                let bytes: usize = self.pending.iter().map(|(_,d)|d.len()).sum();
                if timestamp - since >= 2000 || self.pending.len() > 512 || bytes > 2 * 1024 * 1024 {
                    let first = self.pending.iter().min_by_key(|(s,_)|s.wrapping_sub(next)).unwrap().0;
                    self.next = Some(first); self.gap_since = None; self.losses += 1;
                    continue;
                }
            }
            break;
        }
        out
    }
}

pub struct Pipeline {
    pub storage: Arc<DataStorage>,
    pub engine: PartyEngine,
    assembler: StreamAssembler,
    processor: StreamProcessor,
    order: TcpOrder,
    skills: Arc<SkillLookup>,
    npcs: Arc<NpcLookup>,
    pub timestamp: i64,
    pub packets: u64,
    pub parsed: u64,
    flow: Option<(Option<String>,Option<String>,u16,u16)>,
    completed: Vec<crate::party::Snapshot>,
    pub roster_updates: u64,
    pub current_roster: Vec<Member>,
    pub manual_party: Option<Vec<String>>,
}

impl Pipeline {
    pub fn new() -> Self {
        let storage = Arc::new(DataStorage::new());
        storage.enable_party_feed();
        let skills = Arc::new(SkillLookup::new());
        skills.load_from_json(include_str!("../../src/data/i18n/skills/fr.json"));
        let npcs = Arc::new(NpcLookup::new());
        npcs.load_from_json(include_str!("../../src/data/i18n/npcs/fr.json"));
        let mut processor = StreamProcessor::new(storage.clone(), skills.clone(), npcs.clone());
        let dots: serde_json::Value = serde_json::from_str(include_str!("../../src/data/dot_skill_ids.json")).unwrap();
        let ids = if let Some(a) = dots.as_array() { a.clone() } else {
            dots.get("skills").or_else(||dots.get("dot_skill_ids")).and_then(|v|v.as_array()).cloned().unwrap_or_default()
        };
        processor.set_dot_skill_ids(ids.iter().filter_map(|v|v.as_i64().map(|v|v as i32)).collect());
        Self { storage, engine: PartyEngine::new(), assembler: StreamAssembler::new(), processor,
            order: TcpOrder::default(), skills, npcs, timestamp:0, packets:0, parsed:0, flow:None, completed:vec![],
            roster_updates:0,current_roster:vec![],manual_party:None }
    }

    pub fn feed(&mut self, payload: &CapturedPayload) {
        let flow=(payload.src_ip.clone(),payload.dst_ip.clone(),payload.src_port,payload.dst_port);
        if self.flow.as_ref().is_some_and(|old|old!=&flow) {
            self.engine.boundary("Nouvelle connexion",self.timestamp);
            let mut completed=self.take_finished();
            let mut replacement=Self::new();
            replacement.set_manual_party(self.manual_party.clone());
            *self=replacement;
            self.completed.append(&mut completed);
        }
        self.flow=Some(flow);
        self.timestamp = payload.captured_at_ms;
        self.packets += 1;
        let old_losses = self.order.losses;
        let chunks = self.order.push(payload.tcp_seq, &payload.data, self.timestamp);
        if self.order.losses > old_losses {
            self.engine.transport_incomplete = true;
            self.assembler = StreamAssembler::new();
        }
        for chunk in chunks {
            self.processor.set_override_timestamp(Some(self.timestamp));
            if self.assembler.process_chunk(&chunk, &mut self.processor) { self.parsed += 1; }
            if self.assembler.buffered_bytes() > 2*1024*1024 {
                self.assembler = StreamAssembler::new(); self.engine.transport_incomplete = true;
            }
            self.drain_signals();
            self.processor.set_override_timestamp(None);
        }
        self.engine.tick(self.timestamp);
    }

    pub fn sync_identity(&mut self) {
        let local = self.storage.local_profile();
        let targets: HashMap<_,_> = self.storage.get_mob_data().into_iter().map(|(id,code)| {
            let name = self.npcs.get_npc_name(code);
            (id, if name.is_empty() { format!("Cible #{id}") } else { name })
        }).collect();
        let mobs = targets.keys().copied().collect();
        let bosses = targets.keys().filter(|id| self.storage.is_boss(**id)).copied().collect();
        let id = self.storage.local_player_id().map(|id| id as i32);
        let identity = Identity { names: self.storage.authoritative_names(), local_id:id,
            local_name:local.name.or_else(||self.storage.local_character_name()),
            local_class:class_label(local.class.or_else(||id.and_then(|id|self.storage.actor_job(id)))),
            summons:self.storage.get_summon_data(), summon_ids:self.storage.get_summon_spawn_ids(), mobs, targets, bosses };
        self.engine.sync_identity(identity);
        if let Some(names)=&self.manual_party {
            let identities=self.storage.authoritative_names();
            let members=names.iter().map(|name|Member {key:format!("member:{name}"),name:name.clone(),
                class:class_label(identities.iter().find(|(_,n)|*n==name).and_then(|(id,_)|self.storage.actor_job(*id))),
                present:true,local:false}).collect();
            self.engine.roster(members,true,self.timestamp);
        }
    }

    pub fn set_manual_party(&mut self, names:Option<Vec<String>>) {
        self.manual_party=names;
        if self.manual_party.is_none() {self.engine.roster(self.current_roster.clone(),true,self.timestamp);}
        self.sync_identity();
    }

    /// An explicit click confirms membership, never the player's class or mere
    /// proximity. Keep the other members when switching from decoded to manual.
    pub fn party_with_added_member(&self, name:&str) -> Result<Vec<String>,String> {
        let name=name.trim();
        if name.is_empty() { return Err("Renseigne le pseudo du coéquipier.".into()); }
        let local=self.storage.local_character_name();
        let mut names=self.selected_party_names();
        if local.as_deref()!=Some(name) { names.push(name.to_string()); }
        Ok(validate_manual_party(Some(names))?.unwrap())
    }

    fn selected_party_names(&self)->Vec<String> {
        let local=self.storage.local_character_name();
        self.manual_party.clone().unwrap_or_else(||self.current_roster.iter()
            .filter(|m|m.present&&!m.local&&local.as_deref()!=Some(m.name.as_str())).map(|m|m.name.clone()).collect())
    }
    pub fn party_without_member(&self,name:&str)->Result<Vec<String>,String> {
        let name=name.trim();
        if name.is_empty() {return Err("Pseudo vide.".into());}
        if self.storage.local_character_name().as_deref()==Some(name) {return Err("Utilise le mode solo pour garder uniquement ton personnage.".into());}
        let mut names=self.selected_party_names();names.retain(|n|n!=name);
        Ok(validate_manual_party(Some(names))?.unwrap())
    }

    pub fn observed_players(&self) -> Vec<String> {
        let mobs=self.storage.get_mob_data();let summons=self.storage.get_summon_spawn_ids();
        let local=self.storage.local_player_id().map(|id|id as i32);
        let mut names:Vec<_>=self.storage.authoritative_names().into_iter()
            .filter(|(id,_)|Some(*id)!=local&&!mobs.contains_key(id)&&!summons.contains(id))
            .map(|(_,name)|name).collect();
        names.sort();names.dedup();names
    }

    /// Resume the same observed connection after a short manual stop. Start a
    /// fresh TCP/frame window, preserving identities and roster, not old damage.
    /// `feed` still discards all of this if the actual endpoint tuple changes.
    pub fn resume_capture(&mut self) {
        self.order=TcpOrder::default();self.assembler=StreamAssembler::new();
        self.processor=StreamProcessor::new(self.storage.clone(),self.skills.clone(),self.npcs.clone());
        let ids:serde_json::Value=serde_json::from_str(include_str!("../../src/data/dot_skill_ids.json")).unwrap();
        let ids=ids.as_array().or_else(||ids.get("skills").and_then(|v|v.as_array()))
            .or_else(||ids.get("dot_skill_ids").and_then(|v|v.as_array()));
        self.processor.set_dot_skill_ids(ids.into_iter().flatten().filter_map(|v|v.as_i64().map(|n|n as i32)).collect());
        self.packets=0;self.parsed=0;self.reset_combat();
    }

    pub fn reset_combat(&mut self) {
        self.engine=PartyEngine::new();
        self.engine.roster(self.current_roster.clone(),true,self.timestamp);
        self.sync_identity();
    }

    pub fn drain_signals(&mut self) {
        self.sync_identity();
        for signal in self.storage.take_party_signals() {
            match signal {
                Signal::Roster(members, complete, ts) => {
                    self.roster_updates+=1;
                    if complete {self.current_roster.clear();}
                    for member in &members {
                        self.current_roster.retain(|m|m.key!=member.key && m.name!=member.name);
                        self.current_roster.push(member.clone());
                    }
                    if self.manual_party.is_none() {self.engine.roster(members,complete,ts);}
                },
                Signal::Boundary(reason, ts) => {
                    self.engine.boundary(&reason,ts);
                    // Zone transitions terminate the fight, not the party. The
                    // storage deliberately retains these authoritative identities.
                    if reason=="Changement de zone" {self.engine.roster(self.current_roster.clone(),true,ts);self.sync_identity();}
                },
                Signal::Event(event) if event.kind == EventKind::Death => self.engine.death(event.target,event.timestamp),
                Signal::Event(event) => self.engine.ingest(event),
            }
        }
        // Roster packets and identity packets can be in the same compressed bundle.
        self.sync_identity();
    }

    pub fn snapshot(&self, now: i64) -> crate::party::Snapshot {
        let mut snap = self.engine.snapshot(now);
        self.label_skills(&mut snap); snap
    }
    pub fn dashboard_snapshot(&self, now:i64, detail_member:Option<&str>) -> crate::party::Snapshot {
        self.dashboard_data(now,detail_member,false).0
    }
    pub fn dashboard_data(&self, now:i64, detail_member:Option<&str>, overview:bool) -> (crate::party::Snapshot,Vec<crate::party::Point>) {
        let mut snap=if detail_member.is_some() {self.snapshot(now)}
            else if overview {self.engine.timeline(now)} else {self.engine.summary(now)};
        let mut group=std::collections::BTreeMap::<i64,crate::party::Point>::new();
        if overview {
            for row in &snap.members {
                for point in &row.timeline {
                    let total=group.entry(point.second).or_insert_with(||crate::party::Point{second:point.second,..Default::default()});
                    total.damage+=point.damage;total.healing+=point.healing;total.damage_received+=point.damage_received;
                }
            }
        }
        for row in &mut snap.members {
            if Some(row.member.key.as_str())!=detail_member {row.skills.clear();row.timeline.clear();}
        }
        (snap,group.into_values().collect())
    }
    pub fn label_skills(&self, snap:&mut crate::party::Snapshot) {
        for member in &mut snap.members { for skill in &mut member.skills {
            let name = self.skills.lookup_skill_name(skill.id);
            if !name.is_empty() { skill.name = name; }
        } }
    }
    pub fn losses(&self) -> u64 { self.order.losses }
    pub fn duplicates(&self) -> u64 { self.order.duplicates }
    pub fn take_finished(&mut self) -> Vec<crate::party::Snapshot> {
        let mut completed=std::mem::take(&mut self.completed);
        completed.extend(self.engine.take_finished());
        completed
    }
}

impl Default for Pipeline { fn default()->Self{Self::new()} }

pub fn validate_manual_party(names:Option<Vec<String>>)->Result<Option<Vec<String>>,String> {
    let Some(names)=names else{return Ok(None);};
    let mut result=vec![];
    for name in names {
        let name=name.trim();
        if name.is_empty(){continue;}
        if name.len()>40 || name.chars().any(char::is_control) {return Err("Pseudo invalide (40 octets maximum).".into());}
        if !result.iter().any(|n|n==name){result.push(name.to_string());}
    }
    if result.len()>12 {return Err("12 membres maximum.".into());}
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tcp_overlap_retransmission_and_out_of_order() {
        let mut t = TcpOrder::default();
        assert_eq!(t.push(100,b"abc",0),vec![b"abc".to_vec()]);
        assert!(t.push(106,b"ghi",1).is_empty());
        assert_eq!(t.push(103,b"def",2),vec![b"def".to_vec(),b"ghi".to_vec()]);
        assert!(t.push(100,b"abcdefghi",3).is_empty());
        assert_eq!(t.push(107,b"hijkl",4),vec![b"jkl".to_vec()]);
        assert_eq!(t.losses,0);
    }
    #[test]
    fn tcp_wrap_and_gap_are_bounded() {
        let mut t=TcpOrder::default();
        assert_eq!(t.push(u32::MAX-1,b"ab",0),vec![b"ab".to_vec()]);
        assert_eq!(t.push(0,b"cd",1),vec![b"cd".to_vec()]);
        assert!(t.push(10,b"x",2).is_empty());
        assert_eq!(t.push(11,b"y",2003),vec![b"x".to_vec(),b"y".to_vec()]);
        assert_eq!(t.losses,1);
    }
}
