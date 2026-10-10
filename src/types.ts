export type Metric = 'damage' | 'healing' | 'damageReceived';
export interface Member {key:string;name:string;class:string;present:boolean;local:boolean}
export interface Totals {damage:number;healing:number;healingReceived:number;damageReceived:number;deaths:number;dps:number;hps:number}
export interface Skill {id:number;name:string;kind:string;periodic:boolean;amount:number;impacts:number;critical:number}
export interface Point {second:number;damage:number;healing:number;damageReceived:number}
export interface Row {member:Member;identityConfirmed?:boolean;totals:Totals;contribution:number;skills:Skill[];timeline:Point[]}
export interface Snapshot {schemaVersion:number;id:string;startedAt:number|null;duration:number;active:boolean;endReason:string|null;
 members:Row[];totals:Totals;targets:{id:number;name:string}[];selectedTarget:number|null;warnings:string[]}
export interface Diagnostics {mode:string;status:string;error:string|null;npcap:boolean;administrator:boolean;devices:string[];
 selectedDevice:string|null;connection:string|null;packets:number;parsed:number;duplicates:number;losses:number;recording:string|null;fixture:boolean}
export interface Settings {opacity:number;alwaysOnTop:boolean;width:number;height:number;x:number|null;y:number|null;preferredDevice:string|null;manualParty:string[]|null;compact:boolean}
export interface PartyMemberStatus {member:Member;identityConfirmed:boolean}
export interface Dashboard {snapshot:Snapshot;diagnostics:Diagnostics;settings:Settings;persistenceError:string|null;dataDirectory:string;rosterUpdates:number;observedPlayers:string[];groupTimeline:Point[];partyMembers:PartyMemberStatus[]}
export interface HistoryItem {id:string;startedAt:number;duration:number;damage:number;healing:number;members:number;reason:string}
export const emptyTotals = ():Totals => ({damage:0,healing:0,healingReceived:0,damageReceived:0,deaths:0,dps:0,hps:0});
export const emptySnapshot = ():Snapshot => ({schemaVersion:1,id:'',startedAt:null,duration:1,active:false,endReason:null,members:[],
 totals:emptyTotals(),targets:[],selectedTarget:null,warnings:[]});

export function validateSnapshot(value:unknown):Snapshot {
 const v = value as Snapshot;
 const number = (n:unknown):boolean => typeof n==='number' && Number.isFinite(n) && n>=0;
 const totals = (t:Totals):boolean => !!t && ['damage','healing','healingReceived','damageReceived','deaths','dps','hps'].every(k=>number(t[k as keyof Totals]));
 if (!v || v.schemaVersion!==1 || typeof v.id!=='string' || !number(v.duration) || typeof v.active!=='boolean' ||
     !Array.isArray(v.members) || v.members.length>64 || !totals(v.totals) || !Array.isArray(v.targets) || !Array.isArray(v.warnings) ||
     !v.warnings.every(w=>typeof w==='string') || !v.targets.every(t=>number(t.id)&&typeof t.name==='string') ||
     !v.members.every(r=>r?.member && typeof r.member.key==='string' && typeof r.member.name==='string' && typeof r.member.class==='string' &&
       typeof r.member.present==='boolean' && typeof r.member.local==='boolean' && (r.identityConfirmed===undefined||typeof r.identityConfirmed==='boolean') && totals(r.totals) && number(r.contribution) &&
       Array.isArray(r.skills) && r.skills.every(s=>number(s.id)&&typeof s.name==='string'&&typeof s.kind==='string'&&typeof s.periodic==='boolean'&&number(s.amount)&&number(s.impacts)&&number(s.critical)) &&
       Array.isArray(r.timeline) && r.timeline.every(p=>number(p.second)&&number(p.damage)&&number(p.healing)&&number(p.damageReceived)))) {
   throw new Error('Ce fichier ne contient pas un rapport AionPartyMeter v1 valide.');
 }
 return v;
}
