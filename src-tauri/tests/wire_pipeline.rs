use a2tools_dps_meter_lib::{pipeline::Pipeline,capture::captured_payload::CapturedPayload,combat::data_storage::PartyMember,entity::job_class::JobClass};
fn varint(mut n:u32)->Vec<u8>{let mut v=vec![];loop{let b=(n&127)as u8;n>>=7;v.push(b|if n==0{0}else{128});if n==0{return v;}}}
fn frame(body:&[u8])->Vec<u8>{let mut v=varint(body.len()as u32+4);v.extend_from_slice(body);v}
fn hex(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn payload(data:Vec<u8>,seq:u32,timestamp:i64)->CapturedPayload{CapturedPayload{src_port:7777,dst_port:50000,data,device_name:Some("fixture".into()),captured_at_ms:timestamp,src_ip:Some("192.0.2.1".into()),dst_ip:Some("192.0.2.2".into()),tcp_seq:seq,tcp_ack:0}}
fn setup()->Pipeline{
 let mut p=Pipeline::new();p.storage.set_local_identity_from_game(100,Some("Tank".into()));
 for(id,name)in[(100,"Tank"),(1395,"DPS"),(5041,"Heal"),(400,"Idle")]{p.storage.append_nickname_authoritative(id,name);p.storage.note_player_spawn(id);}
 p.storage.set_party_roster([(100,"Tank",JobClass::Templar),(1395,"DPS",JobClass::Gladiator),(5041,"Heal",JobClass::Cleric),(400,"Idle",JobClass::Chanter)].into_iter().enumerate().map(|(slot,(id,name,job))|(name.into(),PartyMember{slot:slot as u8+1,dbid:id as u64,job:Some(job),..Default::default()})).collect(),true);
 p.storage.append_mob(30001,1);p.drain_signals();p
}
fn damage()->Vec<u8>{let mut b=vec![4,0x38];b.extend(hex("b1ea013600f30a40c0f4007a038000010b199b5f01000000ac52a40d021818"));frame(&b)}
fn ally_heal()->Vec<u8>{let mut b=vec![5,0x38];b.extend(varint(100));b.push(1);b.extend(varint(5041));b.extend(hex("b601032c530bb739"));frame(&b)}

#[test]
fn actual_parser_preserves_ally_heal_recipient_and_multihit_damage(){
 let mut p=setup();let d=damage();let h=ally_heal();p.feed(&payload(d.clone(),100,1000));p.feed(&payload(h,100+d.len()as u32,2000));
 let s=p.snapshot(3000);assert_eq!(s.members.len(),4);assert_eq!(s.totals.damage,1700);assert_eq!(s.totals.healing,7351);
 let tank=s.members.iter().find(|r|r.member.name=="Tank").unwrap();assert_eq!(tank.totals.healing_received,7351);
 let heal=s.members.iter().find(|r|r.member.name=="Heal").unwrap();assert_eq!(heal.totals.healing,7351);assert_eq!(heal.totals.damage,0);
 assert_eq!(s.members.iter().find(|r|r.member.name=="DPS").unwrap().skills[0].impacts,3);
}
#[test]
fn fragmented_compressed_retransmitted_stream_matches_complete_stream(){
 let mut inner=damage();inner.extend(ally_heal());let compressed=lz4_flex::compress(&inner);
 let mut b=vec![255,255];b.extend((inner.len()as u32).to_le_bytes());b.extend(compressed);let data=frame(&b);
 let mut complete=setup();complete.feed(&payload(data.clone(),100,2000));
 let mut split=setup();let cut=data.len()/2;
 split.feed(&payload(data[..cut].to_vec(),100,1900));split.feed(&payload(data[cut..].to_vec(),100+cut as u32,2000));
 split.feed(&payload(data.clone(),100,2100));
 let a=complete.snapshot(3000);let b=split.snapshot(3000);assert_eq!(a.totals.damage,b.totals.damage);assert_eq!(a.totals.healing,b.totals.healing);
 assert_eq!(a.totals.damage,1700);assert_eq!(b.totals.healing,7351);assert_eq!(split.duplicates(),1);
}
#[test]
fn malformed_bundle_does_not_invent_damage(){let mut p=setup();p.feed(&payload(frame(&[255,255,5,0,0,0,1,2,3]),100,1000));assert_eq!(p.snapshot(2000).totals.damage,0);}
#[test]
fn resource_restore_is_not_healing(){let mut p=setup();let mut d=a2tools_dps_meter_lib::entity::damage_packet::ParsedDamagePacket::new();
 d.set_actor_id(5041);d.set_target_id(100);d.set_skill_code(16_990_002);d.set_damage(200);d.set_timestamp(1000);p.storage.append_damage(d);p.drain_signals();assert_eq!(p.snapshot(2000).totals.healing,0);}

#[test]
fn zone_transition_preserves_party_for_the_next_open_world_fight(){
 let mut p=setup();let d=damage();p.feed(&payload(d.clone(),100,1000));
 p.feed(&payload(frame(&[0x23,0x36,0]),100+d.len()as u32,12000));
 assert_eq!(p.snapshot(12000).members.iter().filter(|r|r.member.present).count(),4);
 let zone=frame(&[0x23,0x36,0]);p.feed(&payload(d,100+damage().len()as u32+zone.len()as u32,13000));
 let s=p.snapshot(14000);assert_eq!(s.members.len(),4);assert_eq!(s.totals.damage,1700);
 assert_eq!(p.take_finished().len(),1);
}

#[test]
fn manual_party_counts_selected_healer_zero_damage_member_and_excludes_outsider(){
 let mut p=setup();p.set_manual_party(Some(vec!["Heal".into(),"Idle".into()]));
 let d=damage();p.feed(&payload(d.clone(),100,1000)); // DPS is now outside the selected party.
 assert_eq!(p.snapshot(1000).totals.damage,0);
 // Hostile damage received by the local tank starts the shared encounter.
 let mut hit=a2tools_dps_meter_lib::entity::damage_packet::ParsedDamagePacket::new();
 hit.set_actor_id(30001);hit.set_target_id(100);hit.set_skill_code(1_000_001);hit.set_damage(100);hit.set_timestamp(1500);
 p.storage.append_damage(hit);p.drain_signals();
 p.feed(&payload(ally_heal(),100+d.len()as u32,2000));
 let s=p.snapshot(3000);assert_eq!(s.members.len(),3);assert_eq!(s.totals.healing,7351);assert_eq!(s.totals.damage,0);
 assert_eq!(s.members.iter().map(|r|r.totals.healing).sum::<i64>(),s.totals.healing);
 assert!(s.members.iter().find(|r|r.member.name=="Tank").unwrap().member.present);
 assert_eq!(s.members.iter().find(|r|r.member.name=="Idle").unwrap().totals.damage,0);
 p.reset_combat();assert_eq!(p.snapshot(4000).members.iter().filter(|r|r.member.present).count(),3);
}

#[test]
fn manual_party_waits_for_exact_late_identity_without_guessing_a_class(){
 let mut p=Pipeline::new();p.storage.set_local_identity_from_game(100,Some("Tank".into()));p.storage.append_mob(30001,1);
 p.set_manual_party(Some(vec!["DPS".into(),"Missing".into()]));
 p.feed(&payload(damage(),100,1000));assert_eq!(p.snapshot(1000).totals.damage,0);
 assert!(!p.snapshot(1000).members.iter().find(|r|r.member.name=="DPS").unwrap().identity_confirmed);
 p.storage.append_nickname_authoritative(1395,"DPS");p.storage.note_player_spawn(1395);p.timestamp=2000;p.drain_signals();
 let s=p.snapshot(3000);assert_eq!(s.totals.damage,1700);
 assert!(!s.members.iter().find(|r|r.member.name=="Missing").unwrap().identity_confirmed);
}

#[test]
fn complete_disband_does_not_restore_old_members_on_zone_change(){
 let mut p=setup();p.storage.set_party_roster(vec![],true);p.drain_signals();p.reset_combat();
 assert!(p.current_roster.is_empty());assert!(p.storage.get_party_members().is_empty());
 assert_eq!(p.snapshot(2000).members.len(),1);assert!(p.snapshot(2000).members[0].member.present);
}

fn recorded_roster()->Vec<u8>{
 let s=concat!(
 "02 97 a5 f9 08 00 0b 38 35 30 4b 20 e8 bf 9e e5 88 b7 05 63 28 09 00 00 03 85 4b 01 00 00 00 f6 03 1f 02 00 05 ",
 "0c 01 85 4b 01 00 00 00 f6 03 0f e4 b9 9d e5 b7 9e e4 be 9d e7 84 b6 e5 9c a8 06 00 00 00 32 00 00 00 07 17 00 00 f6 03 f6 03 04 38 39 0d 00 00 00 00 00 00 01 01 ",
 "0e 02 62 64 01 00 00 00 f6 03 02 4d 37 0e 00 00 00 32 00 00 00 5e 16 00 00 f6 03 f6 03 04 6a 03 0d 00 00 00 00 00 00 01 01 ",
 "00 03 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 04 00 00 00 00 00 00 00 00 00 00 00 00 ",
 "00 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 04 00 00 00 00 00 00 00 00 00 00 00 00 ",
 "00 05 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 04 00 00 00 00 00 00 00 00 00 00 00 00");
 frame(&s.split_whitespace().map(|b|u8::from_str_radix(b,16).unwrap()).collect::<Vec<_>>())
}

#[test]
fn flow_is_detected_on_fragmented_compressed_roster_before_any_combat(){
 use a2tools_dps_meter_lib::capture::flow_candidate::FlowCandidate;
 let roster=recorded_roster();let mut body=vec![255,255];body.extend((roster.len()as u32).to_le_bytes());body.extend(lz4_flex::compress(&roster));let data=frame(&body);
 let mut candidate=FlowCandidate::default();let cut=data.len()/2;
 assert!(!candidate.push(payload(data[..cut].to_vec(),100,1000)));
 assert!(candidate.push(payload(data[cut..].to_vec(),100+cut as u32,5000)));
 let mut p=Pipeline::new();for packet in candidate.take_packets(){p.feed(&packet);}
 assert_eq!(p.roster_updates,1);assert_eq!(p.snapshot(5000).members.len(),2);assert_eq!(p.packets,2);
}

#[test]
fn sparse_probe_keeps_precombat_identity_and_retransmissions_do_not_lock(){
 use a2tools_dps_meter_lib::capture::flow_candidate::FlowCandidate;
 let mut c=FlowCandidate::default();let identity=payload(frame(&[0x44,0x36,1,2,3,4]),100,0);let mut seq=100+identity.data.len()as u32;
 assert!(!c.push(identity));let sig=frame(&[0x0e,0,0x36,1]);
 for i in 0..11 {
   let p=payload(sig.clone(),seq,5000+i*100);seq+=sig.len()as u32;
   assert!(!c.push(p.clone()));assert!(!c.push(p));
 }
 assert!(c.push(payload(sig,seq,6100)));let packets=c.take_packets();
 assert_eq!(packets[0].captured_at_ms,0);assert!(!c.truncated);
}

#[test]
fn manual_selection_is_recorded_and_replayed_deterministically(){
 use a2tools_dps_meter_lib::replay::{self,Record};
 let selection=Record::PartySelection{timestamp:1000,names:Some(vec!["DPS".into()])};
 let text=[Record::Header{version:1,fixture:false},selection,Record::Packet{payload:payload(damage(),100,2000)}].iter().map(|r|serde_json::to_string(r).unwrap()).collect::<Vec<_>>().join("\n");
 let (a,_,_)=replay::replay(&text).unwrap();let (b,_,_)=replay::replay(&text).unwrap();
 assert_eq!(a.manual_party,Some(vec!["DPS".into()]));assert_eq!(serde_json::to_value(a.snapshot(2000)).unwrap(),serde_json::to_value(b.snapshot(2000)).unwrap());
}

#[test]
fn candidate_eviction_is_bounded_and_signals_partial_data(){
 use a2tools_dps_meter_lib::capture::flow_candidate::FlowCandidate;
 let mut c=FlowCandidate::default();assert!(!c.push(payload(frame(&[1,2,3]),100,0)));
 assert!(!c.push(payload(frame(&[1,2,3]),104,60001)));
 assert!(c.truncated);assert_eq!(c.take_packets().len(),1);
 let mut c=FlowCandidate::default();assert!(!c.push(payload(frame(&[2,0x97,1,2,3]),100,1000)));
}

fn real_owner_record()->Vec<u8>{
 // Layout observed in a real open-world capture. Name and legion anonymized.
 frame(&hex("048d8cec02787bb800b311250506416c6c696541044775696c010000000000000100"))
}
fn real_owner_damage()->Vec<u8>{
 frame(&hex("04388cec023600b311d016b900d0028800014be94c4801000000c253ba0e01290100"))
}

#[test]
fn open_world_loot_identity_recovers_selected_ally_first_hit(){
 let mut p=Pipeline::new();p.storage.set_local_identity_from_game(100,Some("Tank".into()));
 p.set_manual_party(Some(vec!["AllieA".into()]));let d=real_owner_damage();
 p.feed(&payload(d.clone(),100,1000));assert_eq!(p.snapshot(1000).totals.damage,0);
 p.feed(&payload(real_owner_record(),100+d.len()as u32,2000));
 assert_eq!(p.storage.authoritative_names().get(&2227).map(String::as_str),Some("AllieA"));
 let s=p.snapshot(3000);let ally=s.members.iter().find(|r|r.member.name=="AllieA").unwrap();
 assert!(ally.identity_confirmed);assert!(ally.totals.damage>0);assert_eq!(ally.totals.damage,s.totals.damage);
 assert_eq!(s.started_at,Some(1000));assert_eq!(s.members.len(),2);
}

#[test]
fn loot_owner_identity_does_not_automatically_add_a_stranger_to_party(){
 let mut p=Pipeline::new();p.storage.set_local_identity_from_game(100,Some("Tank".into()));let d=real_owner_damage();
 p.feed(&payload(d.clone(),100,1000));p.feed(&payload(real_owner_record(),100+d.len()as u32,2000));
 assert_eq!(p.storage.authoritative_names().get(&2227).map(String::as_str),Some("AllieA"));
 assert_eq!(p.snapshot(3000).members.len(),1);assert_eq!(p.snapshot(3000).totals.damage,0);
}

#[test]
fn fuzzy_ownership_scan_cannot_confirm_party_identity(){
 let mut p=Pipeline::new();p.storage.set_local_identity_from_game(100,Some("Tank".into()));
 // A valid-looking owner only after another byte is insufficient: the broad
 // legacy scan can retain it for display but cannot drive party accounting.
 let mut bytes=hex("048d8cec02787bb800");bytes.push(1);bytes.extend(hex("b311250506416c6c696541044775696c010000000000000100"));
 p.feed(&payload(frame(&bytes),100,1000));assert!(p.storage.authoritative_names().is_empty());
}

#[test]
fn legacy_replay_uses_explicit_party_but_recorded_selection_takes_precedence(){
 use a2tools_dps_meter_lib::replay::{self,Record};
 let d=real_owner_damage();
 let records=vec![Record::Header{version:1,fixture:false},Record::Packet{payload:payload(d.clone(),100,1000)},
    Record::Packet{payload:payload(real_owner_record(),100+d.len()as u32,2000)}];
 let text=records.iter().map(|r|serde_json::to_string(r).unwrap()).collect::<Vec<_>>().join("\n");
 let (auto,_,_)=replay::replay(&text).unwrap();assert_eq!(auto.snapshot(2000).totals.damage,0);
 let (manual,fights,_)=replay::replay_with_party(&text,Some(vec!["AllieA".into()])).unwrap();
 assert!(manual.snapshot(2000).totals.damage>0);assert_eq!(fights.len(),1);
 let mut records=records;records.insert(1,Record::PartySelection{timestamp:900,names:None});
 let text=records.iter().map(|r|serde_json::to_string(r).unwrap()).collect::<Vec<_>>().join("\n");
 let (recorded,_,_)=replay::replay_with_party(&text,Some(vec!["AllieA".into()])).unwrap();
 assert!(recorded.manual_party.is_none());assert_eq!(recorded.snapshot(2000).totals.damage,0);
 let fixture=include_str!("../../examples/groupe-synthetique.apm.jsonl");
 let (default,_,_)=replay::replay(fixture).unwrap();
 let (with_selection,_,_)=replay::replay_with_party(fixture,Some(vec!["AllieA".into()])).unwrap();
 assert_eq!(serde_json::to_value(default.snapshot(default.timestamp)).unwrap(),serde_json::to_value(with_selection.snapshot(with_selection.timestamp)).unwrap());
}

fn received_hit(body:&str)->Vec<u8>{let mut b=vec![4,0x38];b.extend(hex(body));frame(&b)}
#[test]
fn npc_hits_start_combat_and_count_received_damage_once_with_block_flags(){
 // Actual NPC hit layouts from the upstream packet evidence: Shield Block
 // (74), Iron Wall (116), Regeneration (68), Perfect Block (1).
 let samples=[("ca6f0600ca9804b8c112000102010002ebab530701000000904e4a0100",74),
    ("fe2a0600c68d03b8c112000102100002ebab530701000000904e740100",116),
    ("df7e0600a3b103b0b512000302200d02cbf84e0701000000904e440100",68),
    ("e9230600ee9e03e4cc120001024100021b09580701000000904e010100",1)];
 for (body,amount) in samples {
    let data=received_hit(body);let b=hex(body);let target=a2tools_dps_meter_lib::capture::stream_processor::read_varint(&b,0).value;
    let mut p=Pipeline::new();p.storage.set_local_identity_from_game(target as i64,Some("Tank".into()));
    p.feed(&payload(data.clone(),100,1000));p.feed(&payload(data,100,1500));let s=p.snapshot(2000);
    assert!(s.active);assert_eq!(s.totals.damage_received,amount);assert_eq!(s.totals.damage,0);assert_eq!(s.totals.healing,0);
    assert_eq!(s.members.len(),1);assert_eq!(s.members[0].skills[0].kind,"received");
    assert_eq!(s.members[0].timeline[0].damage_received,amount);assert_eq!(p.duplicates(),1);
 }
}

#[test]
fn received_hits_bind_late_party_target_and_never_add_an_outsider(){
 let data=received_hit("ca6f0600ca9804b8c112000102010002ebab530701000000904e4a0100");
 let mut p=Pipeline::new();p.storage.set_local_identity_from_game(100,Some("Tank".into()));p.set_manual_party(Some(vec!["Ally".into()]));
 p.feed(&payload(data,100,1000));assert_eq!(p.snapshot(1000).totals.damage_received,0);
 p.storage.append_nickname_authoritative(14282,"Ally");p.timestamp=2000;p.sync_identity();
 assert_eq!(p.snapshot(3000).totals.damage_received,74);
 assert_eq!(p.snapshot(3000).members.iter().find(|r|r.member.name=="Ally").unwrap().totals.damage_received,74);
 let outsider=received_hit("fe2a0600c68d03b8c112000102100002ebab530701000000904e740100");
 p.feed(&payload(outsider,100+received_hit("ca6f0600ca9804b8c112000102010002ebab530701000000904e4a0100").len()as u32,3000));
 assert_eq!(p.snapshot(4000).totals.damage_received,74);assert_eq!(p.snapshot(4000).members.len(),2);
}

#[test]
fn npc_periodic_damage_is_received_without_a_player_dot_skill(){
 let mut p=setup();let mut body=vec![5,0x38];body.extend(varint(100));body.push(2);body.extend(varint(30001));body.push(0);
 body.extend(12000121u32.to_le_bytes());body.extend(varint(68));
 p.feed(&payload(frame(&body),100,1000));let s=p.snapshot(2000);
 assert_eq!(s.totals.damage_received,68);assert_eq!(s.totals.damage,0);assert_eq!(s.totals.healing,0);
 assert!(s.members.iter().find(|r|r.member.name=="Tank").unwrap().skills[0].periodic);
}

#[test]
fn dashboard_summary_matches_full_combat_without_serializing_all_details(){
 let mut p=setup();let d=damage();p.feed(&payload(d.clone(),100,1000));p.feed(&payload(ally_heal(),100+d.len()as u32,2000));
 for now in [3000,4000]{
  let full=p.snapshot(now);let summary=p.dashboard_snapshot(now,None);
  assert_eq!(serde_json::to_value(&full.totals).unwrap(),serde_json::to_value(&summary.totals).unwrap());
  for row in &summary.members {let actual=full.members.iter().find(|r|r.member.key==row.member.key).unwrap();
   assert_eq!(serde_json::to_value(&row.totals).unwrap(),serde_json::to_value(&actual.totals).unwrap());
   assert!(row.skills.is_empty()&&row.timeline.is_empty());
  }
  let key=full.members.iter().find(|r|r.member.name=="DPS").unwrap().member.key.clone();
  let detail=p.dashboard_snapshot(now,Some(&key));
  assert!(detail.members.iter().find(|r|r.member.key==key).unwrap().skills.len()>0);
  assert!(detail.members.iter().filter(|r|r.member.key!=key).all(|r|r.skills.is_empty()&&r.timeline.is_empty()));
 }
 p.engine.select_target(Some(900));assert_eq!(p.engine.summary(5000).totals.damage,0);
 p.engine.select_target(None);assert_eq!(p.engine.summary(5000).totals.damage,1700);
 p.engine.finish("Test",5000);assert_eq!(p.take_finished()[0].members.iter().map(|r|r.skills.len()).sum::<usize>(),3);
}

#[test]
fn quick_add_keeps_existing_roster_deduplicates_and_waits_for_real_identity(){
 let mut p=setup();p.storage.append_nickname_authoritative(700,"Outside");
 let names=p.party_with_added_member(" NewAlly ").unwrap();p.set_manual_party(Some(names));
 assert!(p.snapshot(1000).members.iter().any(|r|r.member.name=="DPS"));
 assert!(!p.snapshot(1000).members.iter().any(|r|r.member.name=="Outside"));
 assert!(!p.snapshot(1000).members.iter().find(|r|r.member.name=="NewAlly").unwrap().identity_confirmed);
 let names=p.party_with_added_member("NewAlly").unwrap();assert_eq!(names.iter().filter(|n|*n=="NewAlly").count(),1);
 assert!(p.party_with_added_member("   ").is_err());
 p.storage.append_nickname_authoritative(800,"NewAlly");p.sync_identity();
 assert!(p.snapshot(2000).members.iter().find(|r|r.member.name=="NewAlly").unwrap().identity_confirmed);
 assert!(p.observed_players().contains(&"Outside".to_string()));
 assert!(!p.observed_players().contains(&"Tank".to_string()));
}

#[test]
fn short_capture_restart_keeps_identities_without_reforming_but_new_flow_clears_them(){
 let mut p=setup();p.feed(&payload(damage(),100,1000));assert_eq!(p.snapshot(1000).totals.damage,1700);
 p.engine.finish("Capture arrêtée",1500);p.take_finished();p.resume_capture();
 assert!(p.observed_players().contains(&"DPS".to_string()));
 p.feed(&payload(damage(),900000,2000));assert_eq!(p.snapshot(3000).totals.damage,1700);
 assert!(p.snapshot(3000).members.iter().all(|r|r.identity_confirmed));assert_eq!(p.losses(),0);
 let mut next=payload(damage(),300,4000);next.dst_port+=1;p.feed(&next);
 assert!(p.observed_players().is_empty());assert_eq!(p.snapshot(5000).totals.damage,0);
}

#[test]
fn overview_sends_collective_series_without_all_member_details_and_keeps_filter_coherent(){
 let mut p=setup();let d=damage();p.feed(&payload(d.clone(),100,1000));p.feed(&payload(ally_heal(),100+d.len() as u32,2000));
 let full=p.snapshot(3000);let key=full.members.iter().find(|r|r.member.name=="DPS").unwrap().member.key.clone();
 for selected in [None,Some(key.as_str())] {
  let (snap,series)=p.dashboard_data(3000,selected,true);
  assert_eq!(series.iter().map(|p|p.damage).sum::<i64>(),snap.totals.damage);
  assert_eq!(series.iter().map(|p|p.healing).sum::<i64>(),snap.totals.healing);
  assert!(snap.members.iter().filter(|r|Some(r.member.key.as_str())!=selected).all(|r|r.skills.is_empty()&&r.timeline.is_empty()));
 }
 p.engine.select_target(Some(900));let (snap,series)=p.dashboard_data(4000,None,true);
 assert_eq!(snap.totals.damage,0);assert_eq!(series.iter().map(|p|p.damage).sum::<i64>(),0);
 assert_eq!(series.iter().map(|p|p.healing).sum::<i64>(),full.totals.healing);
 assert!(p.dashboard_data(4000,None,false).1.is_empty());
}

#[test]
fn quick_remove_and_solo_follow_current_selection_without_erasing_combat_history(){
 let mut p=setup();let d=damage();p.feed(&payload(d.clone(),100,1000));
 let before=p.snapshot(1500).totals.damage;
 let names=p.party_without_member("DPS").unwrap();assert!(!names.contains(&"DPS".into()));
 p.timestamp=2000;p.set_manual_party(Some(names));
 assert!(!p.engine.party_members().iter().any(|r|r.member.name=="DPS"));
 assert!(p.snapshot(2000).members.iter().any(|r|r.member.name=="DPS"&&!r.member.present));
 p.feed(&payload(d.clone(),100+d.len() as u32,3000));assert_eq!(p.snapshot(3500).totals.damage,before);
 p.engine.finish("Test",4000);p.take_finished();p.timestamp=4500;p.set_manual_party(Some(vec![]));
 let current=p.engine.party_members();assert_eq!(current.len(),1);assert!(current[0].member.local);
 assert!(p.party_without_member("Tank").is_err());assert!(p.party_without_member(" ").is_err());
 let names=p.party_with_added_member("FutureAlly").unwrap();p.set_manual_party(Some(names));
 assert!(p.engine.party_members().iter().any(|r|r.member.name=="FutureAlly"&&!r.identity_confirmed));
 // Frozen last-fight stats do not masquerade as current group membership.
 assert!(p.snapshot(5000).members.iter().any(|r|r.member.name=="DPS"));
 assert!(!p.engine.party_members().iter().any(|r|r.member.name=="DPS"));
}
