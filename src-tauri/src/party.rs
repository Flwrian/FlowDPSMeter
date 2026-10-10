//! Group-first accounting. Independent of pcap, Tauri and wall-clock time.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::cell::RefCell;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub key: String,
    pub name: String,
    pub class: String,
    pub present: bool,
    pub local: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EventKind { Damage, Heal, Death }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatEvent {
    pub timestamp: i64,
    pub source: i32,
    pub target: i32,
    pub skill: i32,
    pub amount: i64,
    pub kind: EventKind,
    pub periodic: bool,
    pub critical: bool,
    pub impacts: u32,
}

#[derive(Debug, Clone)]
pub enum Signal {
    Event(CombatEvent),
    Roster(Vec<Member>, bool, i64),
    Boundary(String, i64),
}

#[derive(Default, Clone, PartialEq)]
pub struct Identity {
    pub names: HashMap<i32, String>,
    pub local_id: Option<i32>,
    pub local_name: Option<String>,
    pub local_class: String,
    pub summons: HashMap<i32, i32>,
    pub summon_ids: HashSet<i32>,
    pub mobs: HashSet<i32>,
    pub targets: HashMap<i32, String>,
    pub bosses: HashSet<i32>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub damage: i64,
    pub healing: i64,
    pub healing_received: i64,
    pub damage_received: i64,
    pub deaths: u32,
    pub dps: f64,
    pub hps: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub id: i32,
    pub name: String,
    pub kind: String,
    pub periodic: bool,
    pub amount: i64,
    pub impacts: u32,
    pub critical: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub second: i64,
    pub damage: i64,
    pub healing: i64,
    pub damage_received: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    pub member: Member,
    #[serde(default = "confirmed_default")]
    pub identity_confirmed: bool,
    pub totals: Totals,
    pub contribution: f64,
    pub skills: Vec<SkillStats>,
    pub timeline: Vec<Point>,
}
fn confirmed_default() -> bool { true }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct PartyMemberStatus {pub member:Member,pub identity_confirmed:bool}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Target { pub id: i32, pub name: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: u32,
    pub id: String,
    pub started_at: Option<i64>,
    pub duration: f64,
    pub active: bool,
    pub end_reason: Option<String>,
    pub members: Vec<Row>,
    pub totals: Totals,
    pub targets: Vec<Target>,
    pub selected_target: Option<i32>,
    pub warnings: Vec<String>,
}

struct CapturedEvent { event: CombatEvent, eligible: HashSet<String> }
struct Encounter {
    start: i64,
    last: i64,
    members: BTreeMap<String, Member>,
    events: Vec<CapturedEvent>,
    incomplete: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Projection { Summary, Timeline, Full }

pub struct PartyEngine {
    members: BTreeMap<String, Member>,
    identity: Identity,
    // Preserve aliases through an encounter, but never across a zone change.
    bindings: HashMap<i32, String>,
    active: Option<Encounter>,
    last_snapshot: Option<Snapshot>,
    pending: Vec<CapturedEvent>,
    finished: Vec<Snapshot>,
    selected_target: Option<i32>,
    pub transport_incomplete: bool,
    snapshot_cache: RefCell<Option<(Projection, bool, Snapshot)>>,
}

impl Default for PartyEngine {
    fn default() -> Self { Self::new() }
}

impl PartyEngine {
    pub fn new() -> Self {
        Self { members: BTreeMap::new(), identity: Identity::default(), bindings: HashMap::new(),
            active: None, last_snapshot: None, pending: vec![], finished: vec![], selected_target: None,
            transport_incomplete: false, snapshot_cache: RefCell::new(None) }
    }

    pub fn sync_identity(&mut self, identity: Identity) {
        if self.identity == identity { return; }
        self.snapshot_cache.get_mut().take();
        self.identity = identity;
        self.ensure_local();
        self.bind_names();
        // Identity packets may arrive after the first hit. Revisit those hits
        // using the membership that existed at capture time, never today's roster.
        let pending = std::mem::take(&mut self.pending);
        for entry in pending {
            if self.belongs(&entry.event, &entry.eligible) {
                self.accept(entry);
            } else {
                self.pending.push(entry);
            }
        }
    }

    fn ensure_local(&mut self) {
        if let Some(id) = self.identity.local_id {
            let name = self.identity.local_name.clone().unwrap_or_else(|| format!("Vous #{id}"));
            let existing = self.members.values().find(|m| m.name == name && m.key != "local").map(|m| m.key.clone());
            let key = existing.unwrap_or_else(|| "local".into());
            if key != "local" && self.members.contains_key("local") {
                self.members.remove("local");
                for value in self.bindings.values_mut() { if *value == "local" { *value = key.clone(); } }
                if let Some(fight) = &mut self.active {
                    fight.members.remove("local");
                    for entry in &mut fight.events {
                        if entry.eligible.remove("local") { entry.eligible.insert(key.clone()); }
                    }
                }
                for entry in &mut self.pending {
                    if entry.eligible.remove("local") { entry.eligible.insert(key.clone()); }
                }
            }
            let member = self.members.entry(key.clone()).or_insert(Member {
                key: key.clone(), name: name.clone(), class: self.identity.local_class.clone(), present: true, local: true });
            member.local = true;
            member.present = true;
            member.name = name;
            if !self.identity.local_class.is_empty() { member.class = self.identity.local_class.clone(); }
            self.bindings.insert(id, key);
            if let Some(fight) = &mut self.active { fight.members.insert(member.key.clone(), member.clone()); }
        }
    }

    fn bind_names(&mut self) {
        let candidates = self.active.as_ref().map(|f| &f.members).unwrap_or(&self.members);
        for (&id, name) in &self.identity.names {
            let matches: Vec<_> = candidates.values().filter(|m| &m.name == name).collect();
            if matches.len() == 1 { self.bindings.insert(id, matches[0].key.clone()); }
        }
    }

    fn owner(&self, mut id: i32) -> Option<String> {
        let mut seen = HashSet::new();
        while let Some(&owner) = self.identity.summons.get(&id) {
            if !seen.insert(id) { return None; }
            id = owner;
        }
        // An unlinked summon must never inherit a player's name by guesswork.
        if self.identity.summon_ids.contains(&id) { return None; }
        self.bindings.get(&id).cloned()
    }

    fn belongs(&self, event: &CombatEvent, eligible: &HashSet<String>) -> bool {
        self.owner(event.source).is_some_and(|k| eligible.contains(&k))
            || self.owner(event.target).is_some_and(|k| eligible.contains(&k))
    }

    pub fn roster(&mut self, mut roster: Vec<Member>, complete: bool, timestamp: i64) {
        self.tick(timestamp);
        let previous = self.members.clone();
        if complete {
            for member in self.members.values_mut() { member.present = false; }
        }
        for member in &mut roster {
            member.present = true;
            member.local = self.identity.local_name.as_deref() == Some(member.name.as_str());
            self.members.insert(member.key.clone(), member.clone());
        }
        self.ensure_local();
        if let Some(fight) = &mut self.active {
            for member in self.members.values() { fight.members.insert(member.key.clone(), member.clone()); }
        } else { self.members.retain(|_, m| m.present || m.local); }
        self.bind_names();
        if previous != self.members { self.snapshot_cache.get_mut().take(); }
    }

    pub fn ingest(&mut self, event: CombatEvent) {
        if event.amount < 0 { return; }
        self.snapshot_cache.get_mut().take();
        self.tick(event.timestamp);
        let eligible: HashSet<_> = self.members.values().filter(|m| m.present).map(|m| m.key.clone()).collect();
        let entry = CapturedEvent { event, eligible };
        if self.belongs(&entry.event, &entry.eligible) { self.accept(entry); }
        else if !self.identity.names.contains_key(&entry.event.source)
            && !self.identity.names.contains_key(&entry.event.target) {
            self.pending.push(entry);
            // Bound pending identities and memory. Unknown entities never become fake rows.
            if self.pending.len() > 10_000 { self.pending.remove(0); self.transport_incomplete = true; }
        }
    }

    fn accept(&mut self, entry: CapturedEvent) {
        self.snapshot_cache.get_mut().take();
        let e = &entry.event;
        let source = self.owner(e.source).filter(|k| entry.eligible.contains(k));
        let target = self.owner(e.target).filter(|k| entry.eligible.contains(k));
        if e.kind == EventKind::Heal && target.is_none() {
            if source.is_some() && !self.identity.names.contains_key(&e.target) {
                self.pending.push(entry);
                if self.pending.len()>10_000 {self.pending.remove(0);self.transport_incomplete=true;}
            }
            return;
        }
        let hostile = e.kind == EventKind::Damage && e.amount>0 && source.is_some() != target.is_some();
        if self.active.is_none() {
            if !hostile { return; }
            self.active = Some(Encounter { start: e.timestamp, last: e.timestamp,
                members: self.members.iter().filter(|(_, m)| m.present).map(|(k,m)| (k.clone(),m.clone())).collect(),
                events: vec![], incomplete: self.transport_incomplete });
            self.last_snapshot = None;
        }
        let fight = self.active.as_mut().unwrap();
        if e.timestamp < fight.start { return; }
        // Only party healing is included; a stranger's action never creates a row.
        if e.kind == EventKind::Heal && source.is_none() { return; }
        fight.last = fight.last.max(e.timestamp);
        let death = e.kind == EventKind::Death;
        let target_id = e.target;
        if fight.events.len() < 150_000 { fight.events.push(entry); }
        else { fight.incomplete = true; }
        let fought_target = death && self.active.as_ref().unwrap().events.iter().any(|c| c.event.kind==EventKind::Damage && c.event.target==target_id);
        if death && fought_target && (self.selected_target == Some(target_id)
            || (self.selected_target.is_none() && self.identity.bosses.contains(&target_id))) {
            self.finish("Cible vaincue", self.active.as_ref().unwrap().last);
        }
    }

    pub fn death(&mut self, target: i32, timestamp: i64) {
        if self.active.is_none() { return; }
        let eligible = self.active.as_ref().unwrap().members.keys().cloned().collect();
        // Boss deaths are relevant even though the boss isn't a member.
        self.accept(CapturedEvent { event: CombatEvent { timestamp, source: target, target, skill: 0,
            amount: 0, kind: EventKind::Death, periodic: false, critical: false, impacts: 0 }, eligible });
    }

    pub fn tick(&mut self, timestamp: i64) {
        if self.active.as_ref().is_some_and(|f| timestamp - f.last >= 10_000) {
            let last = self.active.as_ref().unwrap().last;
            self.finish("Inactivité", last);
        }
        let pending = self.pending.len();
        self.pending.retain(|e| timestamp - e.event.timestamp < 10_000);
        if pending != self.pending.len() { self.snapshot_cache.get_mut().take(); }
    }

    pub fn finish(&mut self, reason: &str, timestamp: i64) {
        if self.active.is_some() {
            // Archives always contain all targets, regardless of the overlay filter.
            let selected=self.selected_target.take();
            self.snapshot_cache.get_mut().take();
            let mut snap = self.snapshot(timestamp);
            self.selected_target=selected;
            snap.active = false;
            snap.end_reason = Some(reason.into());
            self.finished.push(snap.clone());
            self.last_snapshot = Some(snap);
            self.active = None;
            self.members.retain(|_,m| m.present || m.local);
            self.snapshot_cache.get_mut().take();
        }
    }

    pub fn boundary(&mut self, reason: &str, timestamp: i64) {
        self.snapshot_cache.get_mut().take();
        self.finish(reason, timestamp);
        self.pending.clear();
        self.bindings.clear();
        self.identity = Identity::default();
        self.members.clear();
        self.selected_target = None;
    }

    pub fn select_target(&mut self, target: Option<i32>) { self.selected_target = target; self.snapshot_cache.get_mut().take(); }
    pub fn is_active(&self) -> bool { self.active.is_some() }
    /// Current selection, independent of the frozen last combat or its alumni.
    pub fn party_members(&self) -> Vec<PartyMemberStatus> {
        self.members.values().filter(|m|m.present).map(|m|PartyMemberStatus {
            member:m.clone(),identity_confirmed:self.bindings.values().any(|key|key==&m.key)
        }).collect()
    }
    pub fn take_finished(&mut self) -> Vec<Snapshot> { std::mem::take(&mut self.finished) }

    pub fn snapshot(&self, now: i64) -> Snapshot {
        self.cached_snapshot(now, Projection::Full)
    }

    /// Live tables do not need skill breakdowns or time series on every poll.
    /// Full details remain available for the selected member, history and exports.
    pub fn summary(&self, now: i64) -> Snapshot {
        self.cached_snapshot(now, Projection::Summary)
    }

    /// Group graphs need time buckets, without computing skill breakdowns.
    pub fn timeline(&self, now: i64) -> Snapshot {
        self.cached_snapshot(now, Projection::Timeline)
    }

    fn cached_snapshot(&self, now: i64, projection: Projection) -> Snapshot {
        let cached = self.snapshot_cache.borrow();
        let mut snapshot = if let Some((mode, incomplete, snapshot)) = &*cached {
            if *mode == projection && *incomplete == self.transport_incomplete { Some(snapshot.clone()) } else { None }
        } else { None };
        drop(cached);
        if snapshot.is_none() {
            let value = self.build_snapshot(now, projection);
            *self.snapshot_cache.borrow_mut() = Some((projection, self.transport_incomplete, value.clone()));
            snapshot = Some(value);
        }
        let mut snapshot = snapshot.unwrap();
        if snapshot.active {
            snapshot.duration = ((now - snapshot.started_at.unwrap()).max(1000) as f64) / 1000.0;
            snapshot.totals.dps = snapshot.totals.damage as f64 / snapshot.duration;
            snapshot.totals.hps = snapshot.totals.healing as f64 / snapshot.duration;
            for row in &mut snapshot.members {
                row.totals.dps = row.totals.damage as f64 / snapshot.duration;
                row.totals.hps = row.totals.healing as f64 / snapshot.duration;
            }
        }
        snapshot
    }

    fn build_snapshot(&self, now: i64, projection: Projection) -> Snapshot {
        if self.active.is_none() {
            if let Some(snapshot) = &self.last_snapshot {
                let mut snapshot = snapshot.clone();
                for row in &mut snapshot.members {
                    if projection != Projection::Full { row.skills.clear(); }
                    if projection == Projection::Summary { row.timeline.clear(); }
                }
                return snapshot;
            }
        }
        let started = self.active.as_ref().map(|f| f.start);
        let duration = started.map(|s| ((now - s).max(1000) as f64) / 1000.0).unwrap_or(1.0);
        let members = self.active.as_ref().map(|f| &f.members).unwrap_or(&self.members);
        let mut rows: BTreeMap<String, Row> = members.iter().map(|(k,m)| (k.clone(), Row {
            member: m.clone(), identity_confirmed: self.bindings.values().any(|bound|bound==k),
            totals: Totals::default(), contribution: 0.0, skills: vec![], timeline: vec![] })).collect();
        let mut skills: HashMap<String, BTreeMap<(i32,String,bool), SkillStats>> = HashMap::new();
        let mut points: HashMap<String,BTreeMap<i64,Point>> = HashMap::new();
        let mut targets = BTreeMap::new();
        let mut warnings = vec![];
        let mut unresolved = 0;
        let mut seen_deaths = HashSet::new();
        if let Some(fight) = &self.active {
            for captured in &fight.events {
                let e = &captured.event;
                let source = self.owner(e.source).filter(|k| captured.eligible.contains(k));
                let target = self.owner(e.target).filter(|k| captured.eligible.contains(k));
                if self.identity.summon_ids.contains(&e.source) && source.is_none() { unresolved += 1; }
                if e.kind == EventKind::Death {
                    if seen_deaths.insert((e.target,e.timestamp)) {
                        if let Some(row) = target.as_ref().and_then(|k| rows.get_mut(k)) { row.totals.deaths += 1; }
                    }
                    continue;
                }
                let healing = e.kind == EventKind::Heal || (source.is_some() && target.is_some());
                let mut add = |key: &String, kind: &str, amount: i64| {
                    if let Some(row) = rows.get_mut(key) {
                        match kind {
                            "damage" => row.totals.damage += amount,
                            "heal" => row.totals.healing += amount,
                            "receivedHeal" => row.totals.healing_received += amount,
                            _ => row.totals.damage_received += amount,
                        }
                        if projection == Projection::Summary { return; }
                        if projection == Projection::Full {
                          let skill = skills.entry(key.clone()).or_default().entry((e.skill,kind.into(),e.periodic)).or_insert(SkillStats {
                            id:e.skill, name:format!("Compétence {}",e.skill), kind:kind.into(), periodic:e.periodic, ..Default::default() });
                          skill.amount += amount; skill.impacts += e.impacts.max(1); if e.critical { skill.critical += 1; }
                        }
                        let second = (e.timestamp - fight.start) / 1000;
                        let point = points.entry(key.clone()).or_default().entry(second).or_insert(Point { second,..Default::default() });
                        match kind { "damage" => point.damage += amount, "heal" => point.healing += amount,
                            "received" => point.damage_received += amount, _ => {} }
                    }
                };
                if healing {
                    if let Some(key) = &source { add(key,"heal",e.amount); if let Some(key) = &target { add(key,"receivedHeal",e.amount); } }
                } else {
                    if let Some(key) = &source {
                        targets.insert(e.target, self.identity.targets.get(&e.target).cloned().unwrap_or_else(|| format!("Cible #{}",e.target)));
                        if self.selected_target.is_none() || self.selected_target == Some(e.target) { add(key,"damage",e.amount); }
                    }
                    if let Some(key) = &target { add(key,"received",e.amount); }
                }
            }
            if fight.incomplete || self.transport_incomplete { warnings.push("Capture partielle : certains événements ont été perdus.".into()); }
        }
        let mut totals = Totals::default();
        for (key,row) in &mut rows {
            row.skills = skills.remove(key).unwrap_or_default().into_values().collect();
            row.timeline = points.remove(key).unwrap_or_default().into_values().collect();
            row.totals.dps = row.totals.damage as f64 / duration;
            row.totals.hps = row.totals.healing as f64 / duration;
            totals.damage += row.totals.damage; totals.healing += row.totals.healing;
            totals.healing_received += row.totals.healing_received; totals.damage_received += row.totals.damage_received;
            totals.deaths += row.totals.deaths;
        }
        totals.dps = totals.damage as f64 / duration; totals.hps = totals.healing as f64 / duration;
        for row in rows.values_mut() { row.contribution = if totals.damage > 0 { row.totals.damage as f64 / totals.damage as f64 * 100.0 } else { 0.0 }; }
        if !self.pending.is_empty() || unresolved > 0 { warnings.push(format!("{} événement(s) sans identité ou propriétaire confirmé ; exclus des totaux.",self.pending.len()+unresolved)); }
        if self.identity.local_id.is_none() { warnings.push("Personnage local non identifié : données du groupe potentiellement incomplètes.".into()); }
        if rows.values().any(|r| !self.bindings.values().any(|k| k == &r.member.key)) {
            warnings.push("Certains membres sont connus par le groupe mais leur identité réseau manque encore.".into());
        }
        Snapshot { schema_version:1, id:started.map(|s|format!("combat-{s}")).unwrap_or_default(), started_at:started,
            duration, active:self.active.is_some(), end_reason:None, members:rows.into_values().collect(), totals,
            targets:targets.into_iter().map(|(id,name)|Target{id,name}).collect(), selected_target:self.selected_target, warnings }
    }
}

pub fn class_label(job: Option<crate::entity::job_class::JobClass>) -> String {
    use crate::entity::job_class::JobClass::*;
    match job { Some(Gladiator)=>"Gladiateur",Some(Templar)=>"Templier",Some(Ranger)=>"Rôdeur",
        Some(Assassin)=>"Assassin",Some(Sorcerer)=>"Sorcier",Some(Cleric)=>"Clerc",
        Some(Elementalist)=>"Spiritualiste",Some(Chanter)=>"Aède",Some(Fighter)=>"Combattant",None=>"" }.into()
}
