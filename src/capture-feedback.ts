import type {Diagnostics,PartyMemberStatus,Snapshot} from './types';

export class ActivityTracker {
 private baseline:number|null=null;
 private context='';
 private seen:number|null=null;
 observe(diagnostics:Diagnostics|null,now:number):void {
  const key=diagnostics?`${diagnostics.mode}:${diagnostics.connection??''}`:'';
  if(key!==this.context||diagnostics&&this.baseline!==null&&diagnostics.packets<this.baseline){this.baseline=null;this.seen=null;this.context=key;}
  if(!diagnostics||diagnostics.mode!=='live'){this.baseline=null;this.seen=null;return;}
  if(this.baseline!==null&&diagnostics.packets>this.baseline)this.seen=now;
  this.baseline=diagnostics.packets;
 }
 age(now:number):number|null {return this.seen===null?null:Math.max(0,(now-this.seen)/1000);}
}
export interface Feedback {capture:string;tone:'good'|'muted'|'warning'|'error';combat:string;partial:boolean;activity:boolean;title:string;groupTitle:string;age:string}
export function captureFeedback(d:Diagnostics|null,snapshot:Snapshot,members:PartyMemberStatus[],rosterUpdates:number,age:number|null,report:boolean):Feedback {
 const recordedPartial=snapshot.warnings.some(w=>w.includes('Capture partielle'));
 const partial=recordedPartial||(!report&&(d?.losses??0)>0);
 const missingDriver=!!d&&!d.npcap&&d.mode!=='live'&&d.mode!=='replay';
 const capture=report?'Rapport local':!d?'Visionneuse':d.mode==='replay'?'Replay local':d.error?'Capture à vérifier':missingDriver?'Npcap absent':d.mode==='live'?(d.connection?'Capture active':'Recherche du jeu'):d.mode==='stopped'?'Capture arrêtée':'Capture prête';
 const tone=report||d?.mode==='replay'?'muted':d?.error?'error':missingDriver?'warning':d?.mode==='live'&&d.connection?'good':'muted';
 const combat=snapshot.active?'Combat en cours':snapshot.startedAt!==null?'Combat terminé':'Attente de combat';
 const ageLabel=report||d?.mode!=='live'?'Hors affichage en direct':age===null?'Aucune nouvelle hausse du compteur observée depuis l’ouverture de cette capture':`Derniers paquets observés par l’affichage : il y a ${Math.floor(age)} s`;
 const confirmed=members.filter(m=>m.identityConfirmed).length;
 const title=[capture,d?.status??'',`Connexion : ${d?.connection??'non détectée'}`,`Paquets traités : ${d?.packets??0}`,ageLabel,`Pertes détectées : ${d?.losses??0}`,
  d?.error?`Erreur : ${d.error}`:'',report&&d?.mode==='live'?'La capture continue en arrière-plan ; ces statistiques appartiennent au rapport affiché.':''].filter(Boolean).join('\n');
 const groupTitle=[`${confirmed}/${members.length} identités confirmées`, `Listes de groupe reçues : ${rosterUpdates}`,...members.map(r=>`${r.member.name} : ${r.identityConfirmed?'identifié':'identité en attente'}${r.member.local?' · vous':''}`)].join('\n');
 return {capture,tone,combat,partial,activity:!report&&d?.mode==='live'&&age!==null&&age<.7,title,groupTitle,age:ageLabel};
}
