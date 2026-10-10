use std::collections::{HashMap,HashSet};
use a2tools_dps_meter_lib::party::*;

fn member(key:&str,name:&str,class:&str)->Member{Member{key:key.into(),name:name.into(),class:class.into(),present:true,local:false}}
fn identity()->Identity{Identity{local_id:Some(100),local_name:Some("Tank".into()),local_class:"Templier".into(),
    names:HashMap::from([(100,"Tank".into()),(200,"DPS".into()),(300,"Heal".into()),(400,"Idle".into()),(999,"Stranger".into())]),
    mobs:HashSet::from([800,801]),targets:HashMap::from([(800,"Boss".into()),(801,"Add".into())]),bosses:HashSet::from([800]),..Default::default()}}
fn roster()->Vec<Member>{vec![member("tank","Tank","Templier"),member("dps","DPS","Gladiateur"),member("heal","Heal","Clerc"),member("idle","Idle","Aède")]}
fn engine()->PartyEngine{let mut e=PartyEngine::new();e.sync_identity(identity());e.roster(roster(),true,0);e}
fn hit(timestamp:i64,source:i32,target:i32,amount:i64)->CombatEvent{CombatEvent{timestamp,source,target,skill:11_010_001,amount,kind:EventKind::Damage,periodic:false,critical:false,impacts:1}}
fn heal(timestamp:i64,source:i32,target:i32,amount:i64,periodic:bool)->CombatEvent{CombatEvent{kind:EventKind::Heal,periodic,skill:17_010_001,..hit(timestamp,source,target,amount)}}
fn row<'a>(s:&'a Snapshot,key:&str)->&'a Row{s.members.iter().find(|r|r.member.key==key).unwrap()}

#[test]
fn whole_party_and_common_duration_including_heal_only_and_idle(){
 let mut e=engine();e.ingest(hit(1000,200,800,1000));e.ingest(hit(2000,100,800,500));
 e.ingest(heal(3000,300,100,600,false));e.ingest(heal(4000,300,200,400,true));
 e.ingest(hit(5000,800,100,200));let s=e.snapshot(6000);
 assert_eq!(s.members.len(),4);assert_eq!(s.totals.damage,1500);assert_eq!(s.totals.healing,1000);
 assert_eq!(s.totals.healing_received,1000);assert_eq!(s.totals.damage_received,200);
 assert_eq!(s.duration,5.0);assert_eq!(s.totals.dps,300.0);assert_eq!(row(&s,"dps").totals.dps,200.0);
 assert_eq!(row(&s,"heal").totals.damage,0);assert_eq!(row(&s,"heal").totals.hps,200.0);
 assert_eq!(row(&s,"idle").totals.damage,0);assert_eq!(row(&s,"idle").totals.healing,0);
 assert_eq!(s.totals.damage,s.members.iter().map(|r|r.totals.damage).sum::<i64>());
 assert_eq!(s.totals.healing,s.members.iter().map(|r|r.totals.healing).sum::<i64>());
 assert_eq!(row(&s,"tank").totals.healing_received,600);
 assert_eq!(row(&s,"heal").skills.iter().filter(|s|s.periodic).count(),1);
}
#[test]
fn outsider_is_excluded_even_on_same_boss(){let mut e=engine();e.ingest(hit(1000,200,800,100));
 e.ingest(hit(2000,999,800,10_000));e.ingest(heal(3000,999,100,500,false));let s=e.snapshot(4000);
 assert_eq!(s.totals.damage,100);assert_eq!(s.totals.healing,0);assert_eq!(s.members.len(),4);}
#[test]
fn self_heal_and_pet_are_counted_once(){let mut e=engine();let mut id=identity();id.summons.insert(500,200);id.summon_ids.insert(500);e.sync_identity(id);
 e.ingest(hit(1000,500,800,200));e.ingest(heal(2000,300,300,100,true));let s=e.snapshot(3000);
 assert_eq!(s.members.len(),4);assert_eq!(row(&s,"dps").totals.damage,200);
 assert_eq!(row(&s,"heal").totals.healing_received,100);assert_eq!(s.totals.healing,100);}
#[test]
fn late_summon_owner_recovers_event_without_new_player(){let mut e=engine();let mut id=identity();id.summon_ids.insert(500);e.sync_identity(id.clone());
 e.ingest(hit(1000,200,800,100));e.ingest(hit(1500,500,800,200));assert_eq!(e.snapshot(2000).totals.damage,100);
 id.summons.insert(500,200);e.sync_identity(id);assert_eq!(e.snapshot(2000).totals.damage,300);assert_eq!(e.snapshot(2000).members.len(),4);}
#[test]
fn roster_changes_preserve_departed_members_but_exclude_later_hits(){let mut e=engine();e.ingest(hit(1000,200,800,100));
 let mut new=roster();new.retain(|m|m.key!="dps");new.push(member("new","New","Assassin"));e.roster(new,true,2000);
 let mut id=identity();id.names.insert(600,"New".into());e.sync_identity(id);
 e.ingest(hit(3000,600,800,200));e.ingest(hit(4000,200,800,999));let s=e.snapshot(5000);
 assert_eq!(s.members.len(),5);assert!(!row(&s,"dps").member.present);assert_eq!(row(&s,"dps").totals.damage,100);
 assert_eq!(row(&s,"new").totals.damage,200);assert_eq!(s.totals.damage,300);}
#[test]
fn partial_roster_never_removes_missing_members(){let mut e=engine();e.roster(vec![member("tank","Tank","Templier")],false,500);
 assert_eq!(e.snapshot(500).members.len(),4);assert!(e.snapshot(500).members.iter().all(|r|r.member.present));}
#[test]
fn late_identity_preserves_first_hit_and_heals(){let mut e=PartyEngine::new();let mut id=identity();id.names.remove(&200);e.sync_identity(id);e.roster(roster(),true,0);
 assert!(!row(&e.snapshot(0),"dps").identity_confirmed);
 e.ingest(hit(1000,200,800,1000));assert!(!e.snapshot(1000).active);
 e.sync_identity(identity());let s=e.snapshot(3000);assert_eq!(s.totals.damage,1000);assert_eq!(s.duration,2.0);}
#[test]
fn healing_to_outsiders_is_excluded_and_late_ally_target_is_recovered(){
 let mut e=engine();e.ingest(hit(1000,200,800,100));
 e.ingest(heal(1500,300,999,999,false));assert_eq!(e.snapshot(1800).totals.healing,0);
 let mut id=identity();id.names.remove(&400);e=PartyEngine::new();e.sync_identity(id);e.roster(roster(),true,0);
 e.ingest(hit(1000,200,800,100));e.ingest(heal(2000,300,400,200,false));assert_eq!(e.snapshot(2500).totals.healing,0);
 e.sync_identity(identity());let s=e.snapshot(3000);assert_eq!(s.totals.healing,200);assert_eq!(row(&s,"idle").totals.healing_received,200);
}
#[test]
fn party_disband_does_not_erase_the_fight(){let mut e=engine();e.ingest(hit(1000,200,800,100));e.roster(vec![member("tank","Tank","Templier")],true,2000);
 let s=e.snapshot(3000);assert_eq!(s.members.len(),4);assert_eq!(s.totals.damage,100);assert!(row(&s,"tank").member.present);assert!(!row(&s,"heal").member.present);}
#[test]
fn inactivity_splits_and_keeps_frozen_rates(){let mut e=engine();e.ingest(hit(1000,200,800,100));e.ingest(hit(3000,200,800,100));e.tick(13000);
 let s=e.snapshot(20000);assert!(!s.active);assert_eq!(s.duration,2.0);assert_eq!(s.totals.dps,100.0);assert_eq!(e.take_finished().len(),1);
 e.ingest(hit(21000,200,800,300));assert_eq!(e.snapshot(22000).totals.damage,300);}
#[test]
fn deaths_end_only_the_fought_boss_and_count_player_deaths(){let mut e=engine();e.ingest(hit(1000,200,800,100));
 e.death(100,2000);e.death(100,2000);assert_eq!(e.snapshot(3000).totals.deaths,1);
 e.death(801,3000);assert!(e.is_active());e.death(800,4000);assert!(!e.is_active());assert_eq!(e.take_finished()[0].end_reason.as_deref(),Some("Cible vaincue"));}
#[test]
fn target_filter_does_not_filter_heals_and_archive_keeps_all_damage(){let mut e=engine();e.ingest(hit(1000,200,800,100));e.ingest(hit(2000,200,801,200));e.ingest(heal(3000,300,100,50,false));
 e.select_target(Some(800));let s=e.snapshot(4000);assert_eq!(s.totals.damage,100);assert_eq!(s.totals.healing,50);
 e.finish("Test",4000);assert_eq!(e.take_finished()[0].totals.damage,300);}
#[test]
fn healing_before_combat_does_not_start_a_fight(){let mut e=engine();e.ingest(heal(1000,300,100,100,false));assert!(!e.is_active());
 e.ingest(hit(2000,200,800,200));assert_eq!(e.snapshot(3000).totals.healing,0);}
#[test]
fn solo_has_only_you_and_confirmed_summons(){let mut e=PartyEngine::new();let mut id=identity();id.summons.insert(500,100);id.summon_ids.insert(500);e.sync_identity(id);
 e.ingest(hit(1000,500,800,100));e.ingest(hit(1500,200,800,10_000));let s=e.snapshot(2000);assert_eq!(s.members.len(),1);assert_eq!(s.totals.damage,100);}
#[test]
fn zone_boundary_clears_entity_aliases(){let mut e=engine();e.ingest(hit(1000,200,800,100));e.boundary("Zone",2000);
 assert_eq!(e.take_finished().len(),1);e.sync_identity(Identity{local_id:Some(999),local_name:Some("Stranger".into()),..Default::default()});
 e.ingest(hit(3000,200,800,1000));e.ingest(hit(4000,999,800,50));assert_eq!(e.snapshot(5000).totals.damage,50);}
#[test]
fn local_identity_changes_to_roster_key_without_losing_damage(){let mut e=PartyEngine::new();e.sync_identity(identity());e.ingest(hit(1000,100,800,100));
 e.roster(roster(),true,2000);let s=e.snapshot(3000);assert_eq!(s.members.len(),4);assert_eq!(row(&s,"tank").totals.damage,100);assert!(row(&s,"tank").member.local);}
#[test]
fn replay_refuses_synthetic_records_in_real_capture(){let text="{\"type\":\"header\",\"version\":1,\"fixture\":false}\n{\"type\":\"boundary\",\"timestamp\":1000,\"reason\":\"Test\"}";
 assert!(a2tools_dps_meter_lib::replay::replay(text).is_err());}
#[test]
fn fixture_replay_is_deterministic_and_preserves_all_four_members(){
 let text=include_str!("../../examples/groupe-synthetique.apm.jsonl");
 let (a,fights_a,fixture_a)=a2tools_dps_meter_lib::replay::replay(text).unwrap();
 let (b,fights_b,fixture_b)=a2tools_dps_meter_lib::replay::replay(text).unwrap();
 assert!(fixture_a&&fixture_b);assert_eq!(fights_a.len(),1);assert_eq!(fights_b.len(),1);
 let sa=a.snapshot(a.timestamp);let sb=b.snapshot(b.timestamp);
 assert_eq!(serde_json::to_value(&sa).unwrap(),serde_json::to_value(&sb).unwrap());
 assert_eq!(sa.members.len(),4);assert_eq!(sa.duration,60.0);
 assert_eq!(sa.totals.damage,sa.members.iter().map(|r|r.totals.damage).sum::<i64>());
 assert!(sa.members.iter().any(|r|r.totals.damage==0&&r.totals.healing>0));
}
#[test]
fn snapshot_performance_for_long_fights(){let mut e=engine();for i in 0..50_000{e.ingest(hit(1000+i,200,800,10));}
 let started=std::time::Instant::now();let s=e.snapshot(51000);assert_eq!(s.totals.damage,500_000);assert!(started.elapsed().as_secs_f64()<1.0);}

#[test]
fn live_summary_cache_updates_rates_and_invalidates_on_late_identity_and_loss(){
 let mut e=engine();e.ingest(hit(1000,200,800,100));
 assert_eq!(e.summary(2000).totals.dps,100.0);assert_eq!(e.summary(3000).totals.dps,50.0);
 e.ingest(hit(3500,600,800,400));let mut roster=roster();roster.push(member("new","New","Clerc"));e.roster(roster,true,3600);
 // An event before membership must remain outside, even after identity arrives.
 e.ingest(hit(3700,600,800,50));assert_eq!(e.summary(4000).totals.damage,100);
 let mut id=identity();id.names.insert(600,"New".into());e.sync_identity(id);
 assert_eq!(e.summary(5000).totals.damage,150);
 e.transport_incomplete=true;assert!(e.summary(5000).warnings.iter().any(|w|w.contains("Capture partielle")));
 e.tick(15000);let archived=e.take_finished();assert_eq!(archived[0].totals.damage,150);
 assert!(!archived[0].members.iter().find(|r|r.member.key=="new").unwrap().skills.is_empty());
}

#[test]
fn long_fight_summary_cuts_ipc_size_without_changing_totals(){
 let mut e=engine();for i in 0..50_000{let mut event=hit(1000+i*20,200,800,10);event.skill+=(i%12)as i32;e.ingest(event);}
 let start=std::time::Instant::now();let full=e.snapshot(1_001_000);let full_time=start.elapsed();
 let start=std::time::Instant::now();let summary=e.summary(1_001_000);let summary_time=start.elapsed();
 assert_eq!(serde_json::to_value(&full.totals).unwrap(),serde_json::to_value(&summary.totals).unwrap());
 let full_bytes=serde_json::to_vec(&full).unwrap().len();let summary_bytes=serde_json::to_vec(&summary).unwrap().len();
 assert!(summary_bytes<full_bytes/5);
 let start=std::time::Instant::now();for i in 0..100{std::hint::black_box(e.summary(1_001_000+i));}let cached_time=start.elapsed();
 println!("50k events: full={full_bytes} B ({full_time:?}); summary={summary_bytes} B ({summary_time:?}); 100 cached polls={cached_time:?}");
}

#[test]
fn timeline_projection_keeps_unique_owner_and_all_party_totals_without_skills(){
 let mut e=engine();let mut id=identity();id.summons.insert(500,200);id.summon_ids.insert(500);e.sync_identity(id);
 e.ingest(hit(1000,500,800,120));e.ingest(hit(1500,200,801,180));e.ingest(heal(2000,300,100,90,false));
 e.ingest(hit(2200,800,100,40));e.ingest(hit(2300,999,800,999));
 let mut next=roster();next.retain(|m|m.key!="dps");e.roster(next,true,2400);e.ingest(hit(2500,200,800,999));
 for target in [None,Some(800)] {
  e.select_target(target);let full=e.snapshot(3000);let timeline=e.timeline(3000);
  assert_eq!(serde_json::to_value(&full.totals).unwrap(),serde_json::to_value(&timeline.totals).unwrap());
  assert_eq!(timeline.members.len(),4);assert!(timeline.members.iter().all(|r|r.skills.is_empty()));
  assert!(!row(&timeline,"dps").member.present);
  assert_eq!(timeline.members.iter().flat_map(|r|&r.timeline).map(|p|p.damage).sum::<i64>(),timeline.totals.damage);
  assert_eq!(timeline.members.iter().flat_map(|r|&r.timeline).map(|p|p.healing).sum::<i64>(),90);
  assert_eq!(timeline.members.iter().flat_map(|r|&r.timeline).map(|p|p.damage_received).sum::<i64>(),40);
  assert_eq!(row(&timeline,"idle").totals.damage,0);assert_eq!(row(&timeline,"heal").totals.healing,90);
 }
 e.finish("Test",3000);let frozen=e.timeline(9000);assert!(!frozen.active);
 assert_eq!(frozen.totals.damage,300);assert!(frozen.members.iter().all(|r|r.skills.is_empty()));
 assert!(e.snapshot(9000).members.iter().any(|r|!r.skills.is_empty()));
 assert!(e.summary(9000).members.iter().all(|r|r.timeline.is_empty()));
}
