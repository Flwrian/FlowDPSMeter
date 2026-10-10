import { invoke, isTauri } from '@tauri-apps/api/core';
import { patchHtml,patchText } from './view-dom';
import { chart,groupTimeline,timelineChart } from './chart';
import { renderStatsTable } from './stats-table';
import type { Column } from './stats-table';
import { transformGeometry } from './window-geometry';
import {renderVisualLayout,readLayout,layouts,layoutSize} from './visual-layout';
import {ActivityTracker,captureFeedback} from './capture-feedback';
import {partyStatus} from './party-status';
import {readRefreshRate,pollInterval,pollDelay} from './live-options';
import { emptySnapshot,validateSnapshot } from './types';
import type { Dashboard,Metric,Row,HistoryItem } from './types';
import './party.css';

const native=isTauri();
let snapshot=emptySnapshot();
let dashboard:Dashboard|null=null;
let metric:Metric='damage';
let sort='rate';
let descending=true;
let selectedMember:string|null=null;
let panel:'history'|'settings'|'diagnostics'|'party'|'export'|null=null;
let history:HistoryItem[]=[];
let viewedId:string|null=null;
let imported=false;
let busy=false;
let refreshing:Promise<void>|null=null;
let compact=localStorage.getItem('apm-compact')==='true';
let overview=localStorage.getItem('apm-overview')==='true';
let allSort:Column='damage',allDescending=true;
let graphs=localStorage.getItem('apm-graphs')!=='false';
let graphKey='';
let partyAdding=false;
let refreshRate=readRefreshRate(localStorage.getItem('apm-refresh-rate'));
let layout=readLayout(localStorage.getItem('apm-layout'));
const activity=new ActivityTracker();
let motion=localStorage.getItem('apm-motion')!=='false';
const reducedMotion=matchMedia('(prefers-reduced-motion: reduce)');
let toastTimer:number|undefined;
const labels:Record<Metric,string>={damage:'Dégâts',healing:'Soins',damageReceived:'Dégâts reçus'};
const compactFormatter=new Intl.NumberFormat('fr-FR',{notation:'compact',maximumFractionDigits:1});
const standardFormatter=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:0});
const fmt=(value:number,short=true):string=>(short?compactFormatter:standardFormatter).format(value);
const duration=(seconds:number):string=>`${Math.floor(seconds/60).toString().padStart(2,'0')}:${Math.floor(seconds%60).toString().padStart(2,'0')}`;
const escape=(value:unknown):string=>String(value??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]!));
const date=(timestamp:number|null):string=>timestamp===null?'':new Date(timestamp).toLocaleString('fr-FR');
const colors:Record<string,string>={'Gladiateur':'#edaf63','Templier':'#799cec','Rôdeur':'#74c982','Assassin':'#c495ef','Sorcier':'#ee858d','Clerc':'#efd47c','Spiritualiste':'#68cdcf','Aède':'#a1d6a0','Combattant':'#e99e78'};
const icon=(type:string):string=>({damage:'↗',healing:'+',damageReceived:'◇'}[type]??'•');
const root=document.querySelector<HTMLDivElement>('#app')!;
root.innerHTML=`<main class="shell"><header title="Glisse cette barre pour déplacer la fenêtre"><div class="brand"><span class="logo"><i></i><i></i><i></i></span><div><strong>AionPartyMeter</strong><span>AION 2 · LE GROUPE ENTIER</span></div></div>
 <nav aria-label="Outils"><button data-action="compact" title="Mode compact" aria-label="Mode compact" aria-pressed="false">▤</button><button data-action="history" title="Historique" aria-label="Historique">◷</button><button data-action="settings" title="Réglages" aria-label="Réglages">⚙</button><button data-action="lock" title="Verrouiller — déverrouillage depuis l’icône près de l’horloge" aria-label="Verrouiller">⌑</button><button data-action="minimize" title="Réduire" aria-label="Réduire">−</button><button data-action="close" title="Fermer" aria-label="Fermer">×</button></nav></header>
 <div id="connection" class="connection feedback"><button id="capture-feedback" data-action="diagnostics"><span class="status-dot"></span><span id="capture-label"></span></button><span id="packet-feedback" title="Activité des paquets observée par l’affichage"></span><span id="combat-feedback"></span><button id="partial-feedback" data-action="diagnostics" hidden>Données partielles</button><button id="capture-toggle" data-action="start">Démarrer</button></div><div id="party-context" class="party-context"><button data-action="party"><span id="party-context-label"></span><span class="party-context-add">Gérer +</span></button><select id="display-layout" aria-label="Affichage du meter"><option value="full">Développé</option>${layouts.map(([value,label])=>`<option value="${value}">${label}</option>`).join('')}</select><span id="compact-time"></span></div><div id="group"></div>
 <div class="tabs" role="tablist" aria-label="Statistiques"><button role="tab" data-metric="damage">↗ Dégâts</button><button role="tab" data-metric="healing">+ Soins</button><button role="tab" data-metric="damageReceived">◇ Dégâts reçus</button><button role="tab" data-action="overview">Toutes les stats</button></div>
 <div class="target-line"><span id="group-count"></span><select id="target" aria-label="Filtrer les dégâts par cible"></select></div>
 <section id="rows" class="rows" aria-label="Membres du groupe"></section>
 <section id="mini-meter" class="mini-meter" aria-label="Dégâts, soins et dégâts reçus du groupe" hidden></section>
 <section id="overview" hidden><div class="overview-tools"><span>Toutes les statistiques · durée commune</span><div><button data-action="graphs" aria-pressed="true">Masquer les graphiques</button><button data-action="enlarge">Agrandir la vue ↗</button></div></div><div id="all-table" class="table-scroll"></div><div id="group-graphs" class="group-graphs"></div><p class="graph-note">Courbes du groupe en intervalles de capture · survole une courbe pour les valeurs. Soins rapportés, overheal inconnu ; morts observées uniquement. Les avertissements de capture partielle s’appliquent aussi aux courbes.</p></section><div id="warnings"></div>
 <footer><span id="scope"></span><div><button data-action="party">Groupe</button><button data-action="reset" title="Terminer le combat et préparer le suivant">Nouveau combat</button><button data-action="diagnostics">Diagnostic</button><button data-action="import">Importer</button><button data-action="export">Exporter</button></div></footer>
 <div id="panel"></div><div id="toast" role="status"></div><input type="file" id="file" accept=".json,.jsonl,.txt" hidden/>
 ${(['North','South','East','West','NorthEast','NorthWest','SouthEast','SouthWest'] as const).map(direction=>`<div class="resize-grip resize-${direction}" data-resize="${direction}" title="Glisse pour redimensionner la fenêtre"></div>`).join('')}</main>`;

function toast(message:string,error=false):void {
 const element=document.querySelector<HTMLDivElement>('#toast')!;
 element.textContent=message;element.className=error?'error':'visible';
 window.clearTimeout(toastTimer);toastTimer=window.setTimeout(()=>{element.className='';},6000);
}
function rate(row:Row):number{return metric==='damage'?row.totals.dps:metric==='healing'?row.totals.hps:row.totals.damageReceived/snapshot.duration;}
function percentage(row:Row):number{return snapshot.totals[metric]>0?row.totals[metric]/snapshot.totals[metric]*100:0;}
function sortedRows():Row[]{return [...snapshot.members].sort((a,b)=>{
 let difference=sort==='name'?a.member.name.localeCompare(b.member.name):sort==='total'?a.totals[metric]-b.totals[metric]:rate(a)-rate(b);
 return descending?-difference:difference;
});}
function render():void {
 compact=dashboard?.settings.compact??compact;
 (root.querySelector('.shell') as HTMLElement).dataset.layout=compact?layout:'full';
 document.querySelector<HTMLSelectElement>('#display-layout')!.value=compact?layout:'full';
 const layoutOption=document.querySelector<HTMLSelectElement>('#layout-option');if(layoutOption)layoutOption.value=layout;
 root.querySelector('.shell')!.classList.toggle('compact',compact);
 root.querySelector('.shell')!.classList.toggle('no-motion',!motion||reducedMotion.matches);
 const showOverview=overview&&!compact;
 root.querySelector('.shell')!.classList.toggle('overview',showOverview);
 document.querySelector<HTMLElement>('#overview')!.hidden=!showOverview;
 document.querySelector<HTMLElement>('#rows')!.hidden=showOverview||compact;
 document.querySelector<HTMLElement>('#mini-meter')!.hidden=!compact;
 const compactButton=root.querySelector<HTMLButtonElement>('[data-action="compact"]')!;
 compactButton.setAttribute('aria-pressed',String(compact));compactButton.title=compact?'Afficher toutes les informations':'Mode compact';
 const compactOption=document.querySelector<HTMLInputElement>('#compact-option');if(compactOption)compactOption.checked=compact;
 if(panel!=='settings')document.documentElement.style.setProperty('--opacity',String(dashboard?.settings.opacity??0.96));
 document.documentElement.style.setProperty('--accent',metric==='healing'?'#91d49c':metric==='damageReceived'?'#eead7f':'#70d4cb');
 const diag=dashboard?.diagnostics;
 const members=dashboard?.partyMembers??snapshot.members.filter(r=>r.member.present).map(r=>({member:r.member,identityConfirmed:r.identityConfirmed!==false}));
 const groupState=imported||viewedId?`Rapport · ${snapshot.members.length} membres`:partyStatus(members,dashboard?.settings.manualParty!=null,dashboard?.rosterUpdates??0);
 patchText(document.querySelector('#party-context-label'),groupState);
 patchText(document.querySelector('#compact-time'),snapshot.startedAt?duration(snapshot.duration):'00:00');
 document.querySelector('#compact-time')!.setAttribute('title',snapshot.endReason??'Durée commune de combat');
 const feedback=captureFeedback(diag??null,snapshot,imported||viewedId?snapshot.members.map(r=>({member:r.member,identityConfirmed:r.identityConfirmed!==false})):members,dashboard?.rosterUpdates??0,activity.age(performance.now()),imported||!!viewedId);
 const capture=document.querySelector<HTMLElement>('#capture-feedback')!;
 capture.dataset.tone=feedback.tone;capture.title=feedback.title;patchText(document.querySelector('#capture-label'),feedback.capture);
 const packets=document.querySelector<HTMLElement>('#packet-feedback')!;packets.classList.toggle('receiving',feedback.activity);packets.title=feedback.age;packets.setAttribute('aria-label',feedback.activity?'Nouveaux paquets reçus':feedback.age);
 const combat=document.querySelector<HTMLElement>('#combat-feedback')!;patchText(combat,feedback.combat.replace('Combat en cours','Combat').replace('Combat terminé','Terminé').replace('Attente de combat','Attente'));combat.title=feedback.combat;combat.classList.toggle('in-combat',snapshot.active);
 const partial=document.querySelector<HTMLElement>('#partial-feedback')!;partial.hidden=!feedback.partial;patchText(partial,compact?'Partiel':'Données partielles');partial.title='Des pertes de capture ont été détectées : les statistiques peuvent être incomplètes. Ouvrir le diagnostic.';
 document.querySelector<HTMLElement>('.party-context>button')!.title=feedback.groupTitle;
 const captureToggle=document.querySelector<HTMLButtonElement>('#capture-toggle')!;captureToggle.dataset.action=diag?.mode==='live'?'stop':'start';captureToggle.disabled=!native||busy;patchText(captureToggle,diag?.mode==='live'?'Arrêter':'Démarrer');
 const t=snapshot.totals;
 if(!compact)patchHtml(document.querySelector<HTMLElement>('#group')!,`<div class="encounter"><div><span class="eyebrow">${snapshot.active?'COMBAT EN COURS':snapshot.startedAt?'COMBAT TERMINÉ':'EN ATTENTE D’UN COMBAT'}</span><h1>${snapshot.selectedTarget!==null?escape(snapshot.targets.find(t=>t.id===snapshot.selectedTarget)?.name??'Cible sélectionnée'):'Statistiques du groupe'}</h1></div><div class="timer">${snapshot.startedAt?duration(snapshot.duration):'00:00'}<small>${escape(snapshot.endReason??'Durée commune')}</small></div></div>
 <div class="compact-summary"><span>${labels[metric]} · ${snapshot.members.filter(r=>r.member.present).length} membres</span><strong>${snapshot.startedAt?fmt(metric==='damage'?t.dps:metric==='healing'?t.hps:t.damageReceived/snapshot.duration):'—'}<small> / s</small></strong><span>${fmt(t[metric])} au total</span></div><div class="summary"><article class="primary"><span>DPS du groupe</span><strong>${snapshot.startedAt?fmt(t.dps):'—'}</strong><small>${fmt(t.damage)} dégâts</small></article><article><span>HPS du groupe</span><strong>${snapshot.startedAt?fmt(t.hps):'—'}</strong><small>${fmt(t.healing)} soins prodigués</small></article><article><span>Dégâts reçus</span><strong>${snapshot.startedAt?fmt(t.damageReceived):'—'}</strong><small>${t.deaths} mort${t.deaths>1?'s':''} observée${t.deaths>1?'s':''}</small></article></div>`);
 document.querySelectorAll<HTMLButtonElement>('[data-metric]').forEach(button=>{const selected=!showOverview&&button.dataset.metric===metric;button.classList.toggle('selected',selected);button.setAttribute('aria-selected',String(selected));});
 const overviewButton=document.querySelector<HTMLButtonElement>('[data-action="overview"]')!;
 overviewButton.classList.toggle('selected',showOverview);overviewButton.setAttribute('aria-selected',String(showOverview));
 document.querySelector('#group-count')!.textContent=`${snapshot.members.filter(r=>r.member.present).length} membre(s) présent(s)${snapshot.members.some(r=>!r.member.present)?' · anciens membres conservés':''}`;
 const target=document.querySelector<HTMLSelectElement>('#target')!;
 const options=`<option value="">Toutes les cibles</option>${snapshot.targets.map(t=>`<option value="${t.id}">${escape(t.name)}</option>`).join('')}`;
 if(target.innerHTML!==options)target.innerHTML=options;
 target.value=snapshot.selectedTarget===null?'':String(snapshot.selectedTarget);
 target.disabled=!native||!!viewedId||imported||!snapshot.active||(!showOverview&&metric!=='damage');
 const rows=document.querySelector('#rows')!;
 if(compact){renderVisualLayout(document.querySelector<HTMLElement>('#mini-meter')!,snapshot,metric,motion&&!reducedMotion.matches,layout);}
 else if(showOverview){renderOverview();}
 else if(!snapshot.members.length){
  rows.innerHTML=`<div class="empty"><div class="empty-symbol">${icon(metric)}</div><h2>Ton personnage et tes coéquipiers.</h2><p>${native?'Démarre la capture et combats. Le meter fonctionne aussi en solo. Pour suivre ton groupe, ouvre Groupe et ajoute les coéquipiers découverts ou leur pseudo exact.':'La capture réseau fonctionne dans l’application Windows. Tu peux aussi importer un rapport JSON local ici.'}</p><button class="wide" data-action="party">Choisir les membres du groupe</button></div>`;
 }else{
  renderRows(rows as HTMLElement);

 }
 const warnings=[...snapshot.warnings];
 if(diag?.fixture)warnings.unshift('SCÉNARIO SYNTHÉTIQUE · ces valeurs servent uniquement à vérifier le fonctionnement.');
 if(diag?.losses&&!imported&&!viewedId)warnings.push(`${diag.losses} perte(s) réseau détectée(s) : statistiques partielles.`);
 if(dashboard?.persistenceError)warnings.push(dashboard.persistenceError);
 if(diag?.error)warnings.push(diag.error);
 if(!imported&&!viewedId){
  if(dashboard?.settings.manualParty!==null && dashboard?.settings.manualParty!==undefined)warnings.push('Composition manuelle : vérifie les membres dans Groupe après chaque entrée ou sortie.');
  else if(diag?.mode==='live' && diag.connection && dashboard?.rosterUpdates===0)warnings.push('Liste de groupe non reçue. Consulte Groupe pour voir les membres actuellement suivis et les identités découvertes.');
 }
 patchHtml(document.querySelector<HTMLElement>('#warnings')!,warnings.length?`<details class="warnings"><summary>⚠ ${warnings.length} information(s) sur les données</summary>${warnings.map(w=>`<p>${escape(w)}</p>`).join('')}</details>`:'');
 document.querySelector('#scope')!.textContent=metric==='healing'?'Soins rapportés · overheal inconnu':snapshot.startedAt?'Local · aucune donnée envoyée':'Npcap · capture passive';
 document.querySelector<HTMLButtonElement>('[data-action="reset"]')!.disabled=!native||imported||!!viewedId;
 renderPanel();
}

function renderOverview():void {
 renderStatsTable(document.querySelector<HTMLElement>('#all-table')!,snapshot.members,snapshot.totals,allSort,allDescending);
 const element=document.querySelector<HTMLElement>('#group-graphs')!;element.hidden=!graphs;
 const toggle=document.querySelector<HTMLButtonElement>('[data-action="graphs"]')!;
 toggle.setAttribute('aria-pressed',String(graphs));patchText(toggle,graphs?'Masquer les graphiques':'Afficher les graphiques');
 document.querySelector<HTMLButtonElement>('[data-action="enlarge"]')!.disabled=!native;
 if(!graphs)return;
 const timeline=viewedId||imported?groupTimeline(snapshot.members):dashboard?.groupTimeline??[];
 const key=`${snapshot.id}:${snapshot.selectedTarget}:${snapshot.active}:${timeline.length?'data':'empty'}:${Math.floor(performance.now()/1000)}`;
 if(key===graphKey)return;graphKey=key;
 patchHtml(element,(['damage','healing','damageReceived'] as const).map(kind=>`<article style="--accent:${kind==='healing'?'#91d49c':kind==='damageReceived'?'#eead7f':'#70d4cb'}"><h3>${kind==='damage'?'DPS du groupe':kind==='healing'?'HPS du groupe':'Dégâts reçus / s'}</h3><p>${fmt(snapshot.totals[kind],false)} au total · ${fmt(snapshot.totals[kind]/Math.max(1,snapshot.duration))} / s en moyenne</p>${timelineChart(timeline,kind,snapshot.duration)}</article>`).join(''));
}

function renderRows(rows:HTMLElement):void {
 const focused=document.activeElement;
 if(!rows.querySelector('.table-head')){
  rows.innerHTML='<div class="table-head"><button data-sort="name"></button><button data-sort="rate"></button><button data-sort="total"></button><span>PART</span></div>';
 }
 const head=rows.querySelector<HTMLElement>('.table-head')!;
 for(const [key,label]of [['name','MEMBRE'],['rate',metric==='damage'?'DPS':metric==='healing'?'HPS':'Reçus / s'],['total','TOTAL']]){
  patchText(head.querySelector(`[data-sort="${key}"]`),`${label} ${sort===key?(descending?'↓':'↑'):''}`);
 }
 const existing=new Map(Array.from(rows.querySelectorAll<HTMLButtonElement>('[data-member]')).map(node=>[node.dataset.member!,node]));
 const max=Math.max(...snapshot.members.map(row=>row.totals[metric]),1);
 let previous:Element=head;
 for(const [index,row]of sortedRows().entries()){
  let node=existing.get(row.member.key);
  if(!node){
   node=document.createElement('button');node.dataset.member=row.member.key;node.className='player-row';
   node.innerHTML='<span class="row-bar"></span><div class="identity"><span class="rank"></span><span class="class-icon"></span><div><strong class="member-name"></strong><small class="member-class"></small></div></div><strong class="rate"></strong><span class="amount"></span><span class="share"></span>';
  }
  node.classList.toggle('departed',!row.member.present);node.setAttribute('aria-label',`Détails de ${row.member.name}`);
  node.style.setProperty('--class-color',colors[row.member.class]??'#899aaa');node.style.setProperty('--bar',`${row.totals[metric]/max*100}%`);
  patchText(node.querySelector('.rank'),String(index+1));patchText(node.querySelector('.class-icon'),row.member.class.slice(0,1)||'?');
  patchText(node.querySelector('.member-name'),row.member.name+(row.member.local?' · vous':''));
  patchText(node.querySelector('.member-class'),(row.member.class||'Classe inconnue')+(row.identityConfirmed===false?' · identité en attente':!row.member.present?' · a quitté le groupe':''));
  patchText(node.querySelector('.rate'),row.identityConfirmed===false?'—':fmt(rate(row)));
  patchText(node.querySelector('.amount'),row.identityConfirmed===false?'—':fmt(row.totals[metric]));
  patchText(node.querySelector('.share'),row.identityConfirmed===false?'—':`${percentage(row).toFixed(1)} %`);
  if(previous.nextElementSibling!==node)rows.insertBefore(node,previous.nextElementSibling);
  previous=node;existing.delete(row.member.key);
 }
 for(const node of existing.values())node.remove();
 if(focused instanceof HTMLButtonElement&&rows.contains(focused)&&document.activeElement!==focused)focused.focus({preventScroll:true});
}

function renderDiscoveries():void {
 const element=document.querySelector<HTMLElement>('#party-discoveries');if(!element)return;
 const members=dashboard?.partyMembers??snapshot.members.filter(r=>r.member.present).map(r=>({member:r.member,identityConfirmed:r.identityConfirmed!==false}));
 const selected=new Set(dashboard?.settings.manualParty??members.filter(r=>!r.member.local).map(r=>r.member.name));
 const local=members.find(r=>r.member.local)?.member.name;
 const names=(dashboard?.observedPlayers??[]).filter(name=>name!==local);
 patchHtml(element,names.map(name=>`<button data-add-player="${escape(name)}" ${selected.has(name)||partyAdding?'disabled':''}>${selected.has(name)?'✓':'+ Ajouter'} ${escape(name)}</button>`).join('')||'<span class="muted">Recherche en direct… les identités apparaissent dès leur réception.</span>');
 patchText(document.querySelector('#party-mode'),`Mode actuel : ${dashboard?.settings.manualParty!=null?'composition manuelle':'détection automatique'} · ${dashboard?.rosterUpdates??0} liste(s) décodée(s)`);
 const followed=document.querySelector<HTMLElement>('#party-followed');
 if(followed)patchHtml(followed,members.map(r=>`<div class="followed-member"><div><strong>${escape(r.member.name)}${r.member.local?' · vous':''}</strong><small class="${r.identityConfirmed?'confirmed':'pending'}">${r.identityConfirmed?'Identité confirmée': 'Identité réseau en attente'}</small></div>${r.member.local?'':`<button data-remove-player="${escape(r.member.name)}" title="Retirer du suivi" ${partyAdding||!native?'disabled':''}>Retirer ×</button>`}</div>`).join('')||'<p class="muted">Le personnage local apparaîtra dès son identification.</p>');
 patchText(document.querySelector('#party-current-status'),partyStatus(members,dashboard?.settings.manualParty!=null,dashboard?.rosterUpdates??0));
}

function renderPanel():void {
 const element=document.querySelector<HTMLElement>('#panel')!;
 const view=selectedMember?`details:${selectedMember}`:panel??'';
 if(element.dataset.view!==view){patchHtml(element,'');element.removeAttribute('data-panel');element.dataset.view=view;}
 if(panel==='settings'||panel==='party') {if(element.getAttribute('data-panel')===panel){if(panel==='party')renderDiscoveries();return;}}
 if(selectedMember){
  const row=snapshot.members.find(r=>r.member.key===selectedMember);
  if(!row){selectedMember=null;renderPanel();return;}
  const kind=metric==='damage'?'damage':metric==='healing'?'heal':'received';
  const skills=row.skills.filter(s=>s.kind===kind).sort((a,b)=>b.amount-a.amount);
  patchHtml(element,`<div class="backdrop"><section class="sheet"><div class="sheet-head"><div><span class="eyebrow">${escape(row.member.class||'MEMBRE DU GROUPE')}</span><h2>${escape(row.member.name)}</h2></div><button data-action="dismiss" aria-label="Fermer les détails">×</button></div><div class="tabs detail-tabs" aria-label="Détails par statistique">${(["damage","healing","damageReceived"] as const).map(kind=>`<button data-detail-metric="${kind}" class="${metric===kind?"selected":""}">${labels[kind]}</button>`).join("")}</div><div class="detail-stats">${[
   ['Dégâts',fmt(row.totals.damage)],['DPS',fmt(row.totals.dps)],['Soins prodigués',fmt(row.totals.healing)],['HPS',fmt(row.totals.hps)],['Soins reçus',fmt(row.totals.healingReceived)],['Dégâts reçus',fmt(row.totals.damageReceived)],['Morts observées',row.totals.deaths],['Contribution dégâts',`${row.contribution.toFixed(1)} %`],
  ].map(([label,value])=>`<div><span>${label}</span><strong>${row.identityConfirmed===false?'—':value}</strong></div>`).join('')}</div><h3>${labels[metric]} par seconde</h3>${chart(row,metric,snapshot.duration)}<h3>Détails par compétence <small>Impacts, pas lancements</small></h3><div class="skill-head"><span>COMPÉTENCE</span><span>IMPACTS</span><span>CRIT.</span><span>TOTAL</span></div>${skills.length?skills.map(s=>`<div class="skill-row"><div>${escape(s.name)}${s.periodic?'<small>Effet périodique</small>':''}</div><span>${s.impacts}</span><span>${s.kind==='heal'?'—':s.critical}</span><strong>${fmt(s.amount)}</strong></div>`).join(''):`<p class="muted">${row.identityConfirmed===false?'Identité réseau en attente : statistiques inconnues.':'Aucun événement pour cette statistique.'}</p>`}<p class="muted">${row.member.present?'Membre présent':'Membre ayant quitté le groupe'} · même durée de combat que tous les autres membres.</p></section></div>`);
  element.setAttribute('data-panel','details');return;
 }
 if(!panel){patchHtml(element,'');element.removeAttribute('data-panel');return;}
 let content='';
 if(panel==='history'){
  content=`<p class="muted">Les 200 derniers combats sont conservés sur ce PC.</p>${history.length?history.map(h=>`<div class="history-row"><button data-history="${escape(h.id)}"><strong>${escape(date(h.startedAt))}</strong><small>${h.members} membres · ${duration(h.duration)} · ${fmt(h.damage)} dégâts · ${fmt(h.healing)} soins</small></button><button data-delete="${escape(h.id)}" aria-label="Supprimer ce combat">×</button></div>`).join(''):'<div class="empty-small">Aucun combat sauvegardé.</div>'}${viewedId||imported?'<button class="wide" data-action="live">Revenir au direct</button>':''}`;
 }else if(panel==='party'){
  const names=dashboard?.settings.manualParty??dashboard?.partyMembers?.filter(r=>!r.member.local).map(r=>r.member.name)??[];
  content=`<p class="muted">Le meter fonctionne aussi en solo. Tes coéquipiers n’ont rien à installer. Pour le groupe, choisis ceux à suivre ici ; une liste réseau reconnue les sélectionne automatiquement en mode Auto.</p><div class="party-mode-actions"><button data-action="auto-party" ${!native?'disabled':''}>Auto</button><button data-action="solo-party" ${!native?'disabled':''}>Solo</button><strong id="party-current-status"></strong></div><h3>Membres suivis maintenant</h3><div id="party-followed"></div><h3>Ajouter un coéquipier</h3><div id="party-discoveries" class="actions discoveries"></div><label>Pseudo exact<div class="quick-add"><input id="party-quick-name" placeholder="Viktor" maxlength="40"/><button class="primary-button" data-action="quick-party" ${!native?'disabled':''}>Ajouter</button></div></label><p class="muted">Les joueurs proches peuvent être découverts sans faire partie de ton groupe. Ajoute seulement tes coéquipiers. Les identités non reçues restent « — » : combats ensemble, ramassez du butin ou rapproche-toi du membre pour recevoir son identité. Tu n’as pas besoin de quitter et recréer le groupe.</p><details class="party-advanced"><summary>Composition complète et diagnostic</summary><p id="party-mode" class="muted"></p><label>Un pseudo par ligne<textarea id="party-names" rows="4">${escape(names.join('\n'))}</textarea></label><button class="wide" data-action="save-party" ${!native?'disabled':''}>Appliquer</button><p class="muted">La sélection est mémorisée. Une identité seule ne prouve pas l’appartenance au groupe. Retirer un membre conserve sa participation passée au combat, mais exclut ses prochains événements. En mode Auto, une liste réseau reconnue sélectionne les membres. Si la liste ou une identité manque, la sélection explicite reste disponible.</p></details>`;

 }else if(panel==='settings'){
  content=`<label class="check"><input id="compact-option" type="checkbox" ${compact?'checked':''}/> Mode compact — dégâts, soins et reçus ensemble</label><label>Affichage compact<select id="layout-option">${layouts.map(([value,label])=>`<option value="${value}" ${layout===value?'selected':''}>${label}</option>`).join('')}</select></label><label>Actualisation des statistiques<select id="refresh-rate">${([4,10,20] as const).map(hz=>`<option value="${hz}" ${refreshRate===hz?'selected':''}>${hz} fois / seconde${hz===10?' · recommandé':hz===20?' · plus de CPU':' · économique'}</option>`).join('')}</select></label><label class="check"><input id="motion-option" type="checkbox" ${motion?'checked':''}/> Animations discrètes</label><p class="muted">Actualisation selon les événements reçus. Les détails de compétences sont limités à 4 mises à jour/s ; la fenêtre masquée à 1/s. Les animations respectent la réduction des mouvements de Windows.</p><div class="actions"><button data-action="import">Importer</button><button data-action="export">Exporter</button><button data-action="reset">Nouveau combat</button><button data-action="diagnostics">Diagnostic</button></div><p class="muted">Déplace la fenêtre en glissant sa barre de titre. Redimensionne-la par les bords ou la poignée en bas à droite. La position et la taille sont mémorisées.</p><button class="wide" data-action="recenter" ${!native?'disabled':''}>Recentrer la fenêtre et rétablir sa taille</button><label>Opacité de l’overlay <input id="opacity" type="range" min="25" max="100" value="${Math.round((dashboard?.settings.opacity??.96)*100)}"/></label><label class="check"><input id="always-top" type="checkbox" ${dashboard?.settings.alwaysOnTop??true?'checked':''}/> Toujours au-dessus du jeu</label><label>Interface réseau<select id="device"><option value="">Détection automatique</option>${dashboard?.diagnostics.devices.map(d=>`<option ${dashboard?.settings.preferredDevice===d?'selected':''}>${escape(d)}</option>`).join('')??''}</select></label><button class="wide primary-button" data-action="save-settings" ${!native?'disabled':''}>Enregistrer les réglages</button><p class="muted">Le verrouillage laisse passer les clics vers le jeu. Pour déverrouiller : icône AionPartyMeter près de l’horloge → Afficher et déverrouiller.</p><hr/><p class="muted">Interface française · fonctionnement local · capture passive Npcap.<br/>AionPartyMeter 0.1.8 · moteur dérivé d’A2Tools, GPL-3.0.</p>`;
 }else if(panel==='export'){
  content=`<p class="muted">${snapshot.members.length} membres · ${duration(snapshot.duration)} · données locales</p><div class="actions"><button data-format="json">Rapport JSON complet</button><button data-format="csv">Tableau CSV</button></div>`;
 }else{
  const d=dashboard?.diagnostics;
  content=`<dl class="diagnostics">${[
   ['Npcap',d?.npcap?'Disponible':'Absent ou non accessible'],['Permissions',d?.administrator?'Administrateur':'Utilisateur standard'],['Connexion',d?.connection??'Non détectée'],['Listes de groupe décodées',dashboard?.rosterUpdates??0],['Identités de joueurs reçues',dashboard?.observedPlayers.length??0],['Activité observée',captureFeedback(d??null,snapshot,dashboard?.partyMembers??[],dashboard?.rosterUpdates??0,activity.age(performance.now()),imported||!!viewedId).age],['Paquets traités',d?.packets??0],['Segments décodés',d?.parsed??0],['Retransmissions ignorées',d?.duplicates??0],['Pertes détectées',d?.losses??0],['Enregistrement',d?.recording??'Arrêté'],['Données locales',dashboard?.dataDirectory??'Visionneuse navigateur'],
  ].map(([k,v])=>`<dt>${k}</dt><dd>${escape(v)}</dd>`).join('')}</dl><div class="actions"><button data-action="record" ${!native||d?.mode!=='live'?'disabled':''}>${d?.recording?'Terminer l’enregistrement':'Enregistrer les paquets'}</button><button data-action="folder" ${!native?'disabled':''}>Dossier local</button></div><p class="muted">Pour un replay complet, commence l’enregistrement avant de former le groupe ou de changer de zone. Les captures brutes peuvent contenir des noms et des messages du jeu ; elles restent sur ce PC.</p><h3>Si rien n’apparaît</h3><ol class="help"><li>Installer Npcap depuis npcap.com.</li><li>Démarrer la capture ; si l’accès est refusé, relancer l’application en administrateur.</li><li>Ouvrir Groupe pour sélectionner les coéquipiers ; aucune sortie de groupe n’est nécessaire.</li><li>Combattre puis vérifier les informations de données partielles.</li></ol><p class="muted">La compatibilité Global / Europe doit encore être confirmée sur un combat réel en groupe.</p>`;
 }
 patchHtml(element,`<div class="backdrop"><section class="sheet"><div class="sheet-head"><h2>${panel==='history'?'Historique':panel==='settings'?'Réglages':panel==='party'?'Composition du groupe':panel==='export'?'Exporter le combat':'Diagnostic de capture'}</h2><button data-action="dismiss" aria-label="Fermer">×</button></div>${content}</section></div>`);
 element.setAttribute('data-panel',panel);if(panel==='party')renderDiscoveries();
}

async function addPartyMember(name:string):Promise<void>{
 if(!native){toast('Ajout de membres disponible dans l’application Windows.');return;}
 if(partyAdding)return;
 partyAdding=true;renderDiscoveries();
 try{
  await invoke('party_add',{name});await refresh(true);
  const input=document.querySelector<HTMLInputElement>('#party-quick-name');if(input)input.value='';
  const composition=document.querySelector<HTMLTextAreaElement>('#party-names');if(composition)composition.value=(dashboard?.settings.manualParty??[]).join('\n');
  toast(`${name.trim()} ajouté au groupe.`);
 }finally{partyAdding=false;renderDiscoveries();}
}
async function removePartyMember(name:string):Promise<void>{
 if(!native||partyAdding)return;
 partyAdding=true;renderDiscoveries();
 try{await invoke('party_remove',{name});await refresh(true);updatePartyDraft();toast(`${name} retiré du suivi. Sa participation au combat reste conservée.`);}
 finally{partyAdding=false;renderDiscoveries();}
}
function updatePartyDraft():void {
 const field=document.querySelector<HTMLTextAreaElement>('#party-names');if(field)field.value=(dashboard?.settings.manualParty??dashboard?.partyMembers?.filter(r=>!r.member.local).map(r=>r.member.name)??[]).join('\n');
}
async function setCompact(value:boolean):Promise<void>{
 if(value){overview=false;localStorage.setItem('apm-overview','false');}
 if(native){await invoke('compact_set',{compact:value});await refresh(true);}
 else{compact=value;localStorage.setItem('apm-compact',String(compact));render();}
}

async function chooseLayout(value:string):Promise<void>{
 if(value==='full'){await setCompact(false);return;}
 layout=readLayout(value);localStorage.setItem('apm-layout',layout);
 if(!compact)await setCompact(true);else render();
 if(native){
  const geometry=await invoke<{x:number;y:number;width:number;height:number;scale:number}>('window_geometry');
  const size=layoutSize(layout,snapshot.members.length);
  await invoke('window_transform',{geometry:{...geometry,width:Math.round(size.width*geometry.scale),height:Math.round(size.height*geometry.scale)},resize:true});
 }
}

async function action(name:string):Promise<void>{
 if(name==='overview'){
  overview=true;metric='damage';graphKey='';localStorage.setItem('apm-overview','true');
  if(compact)await setCompact(false);
  if(native&&window.innerWidth<1050)await invoke('window_action',{action:'overview'});
  if(native)await refresh(true);else render();return;
 }
 if(name==='graphs'){graphs=!graphs;graphKey='';localStorage.setItem('apm-graphs',String(graphs));render();return;}
 if(name==='enlarge'){if(native)await invoke('window_action',{action:'overview'});return;}
 if(name==='compact'){if(compact)await setCompact(false);else await chooseLayout(layout);return;}
 if(name==='quick-party'){await addPartyMember(document.querySelector<HTMLInputElement>('#party-quick-name')!.value);return;}
 if(name==='dismiss'){panel=null;selectedMember=null;renderPanel();return;}
 if(name==='history'){history=native?await invoke('history_list'):[];panel='history';selectedMember=null;renderPanel();return;}
 if(name==='settings'||name==='diagnostics'||name==='party'){panel=name;selectedMember=null;renderPanel();return;}
 if(name==='import'){document.querySelector<HTMLInputElement>('#file')!.click();return;}
 if(name==='live'){viewedId=null;imported=false;panel=null;await refresh();render();return;}
 if(name==='export'){
  if(!snapshot.startedAt){toast('Aucun combat à exporter.');return;}
  panel='export';selectedMember=null;renderPanel();return;
 }
 if(!native){toast('Cette fonction est disponible dans l’application Windows.');return;}
 if(name==='save-party'||name==='auto-party'||name==='solo-party'){
  const names=name==='auto-party'?null:name==='solo-party'?[]:document.querySelector<HTMLTextAreaElement>('#party-names')!.value.split(/\r?\n/);
  await invoke('party_save',{names});await refresh(true);updatePartyDraft();toast(names===null?'Détection automatique rétablie.':names.length?'Composition appliquée. Les identités manquantes restent en attente.':'Mode solo : ton personnage et ses invocations uniquement.');return;
 }
 if(name==='start'){
  const device=document.querySelector<HTMLSelectElement>('#device')?.value??dashboard?.settings.preferredDevice??'';
  busy=true;render();try{await invoke('capture_start',{device:device||null});viewedId=null;imported=false;}finally{busy=false;}await refresh();
 }else if(name==='stop'){await invoke('capture_stop');await refresh();}
 else if(name==='reset'){await invoke('new_combat');await refresh();}
 else if(name==='record'){await invoke('recording_toggle');await refresh();}
 else if(name==='folder'){await invoke('open_data_folder');}
 else if(name==='recenter'){await invoke('window_action',{action:'recenter'});toast('Fenêtre recentrée ; taille et position seront conservées.');}
 else if(name==='save-settings'){
  if(!dashboard)return;
  const opacity=Number(document.querySelector<HTMLInputElement>('#opacity')!.value)/100;
  const alwaysOnTop=document.querySelector<HTMLInputElement>('#always-top')!.checked;
  const preferredDevice=document.querySelector<HTMLSelectElement>('#device')!.value||null;
  await invoke('settings_save',{settings:{...dashboard.settings,opacity,alwaysOnTop,preferredDevice}});
  toast('Réglages enregistrés. L’interface choisie sera utilisée au prochain démarrage de capture.');
  if(dashboard.diagnostics.mode==='live')toast('Pour changer d’interface réseau, arrête puis redémarre la capture.');
  dashboard.settings.opacity=opacity;dashboard.settings.alwaysOnTop=alwaysOnTop;dashboard.settings.preferredDevice=preferredDevice;render();
 }else if(['close','minimize','lock'].includes(name)){
  if(name==='lock'){panel=null;selectedMember=null;renderPanel();toast('Déverrouillage depuis l’icône près de l’horloge.');}
  await invoke('window_action',{action:name});
 }
}

root.addEventListener('click',event=>{
 const element=(event.target as HTMLElement).closest<HTMLElement>('button');
 if(!element)return;
 const run=async()=>{
  if(element.dataset.compactSort){metric=element.dataset.compactSort as Metric;sort='total';descending=true;render();}
  else if(element.dataset.detailMetric){metric=element.dataset.detailMetric as Metric;renderPanel();}
  else if(element.dataset.metric){overview=false;localStorage.setItem('apm-overview','false');metric=element.dataset.metric as Metric;sort='rate';descending=true;render();}
  else if(element.dataset.column){const key=element.dataset.column as Column;if(allSort===key)allDescending=!allDescending;else{allSort=key;allDescending=key!=='name'&&key!=='class';}render();}
  else if(element.dataset.addPlayer){await addPartyMember(element.dataset.addPlayer);}
  else if(element.dataset.removePlayer){await removePartyMember(element.dataset.removePlayer);}
  else if(element.dataset.member){if(element.dataset.panelMetric)metric=element.dataset.panelMetric as Metric;selectedMember=element.dataset.member;panel=null;if(native&&!imported&&!viewedId)await refresh(true);else renderPanel();}
  else if(element.dataset.sort){if(sort===element.dataset.sort)descending=!descending;else{sort=element.dataset.sort;descending=sort!=='name';}render();}
  else if(element.dataset.history){snapshot=await invoke('history_load',{id:element.dataset.history});viewedId=element.dataset.history;imported=false;panel=null;selectedMember=null;render();}
  else if(element.dataset.delete){await invoke('history_delete',{id:element.dataset.delete});history=await invoke('history_list');renderPanel();}
  else if(element.dataset.format){
   const format=element.dataset.format;
   if(native&&!imported){const path=await invoke<string>('export_combat',{id:viewedId,format});toast(`Export sauvegardé : ${path}`);}
   else{
    let text=JSON.stringify(snapshot,null,2);
    if(format==='csv'){
     const q=(s:string)=>`"${(/^[=+\-@\t\r]/.test(s)?"'":'')+s.replace(/"/g,'""')}"`;
     text='\uFEFFNom,Classe,Dégâts,DPS,Soins,HPS,Soins reçus,Dégâts reçus,Morts\r\n'+snapshot.members.map(r=>[q(r.member.name),q(r.member.class),...(r.identityConfirmed===false?['','','','','','','']:[r.totals.damage,r.totals.dps,r.totals.healing,r.totals.hps,r.totals.healingReceived,r.totals.damageReceived,r.totals.deaths])].join(',')).join('\r\n');
    }
    const url=URL.createObjectURL(new Blob([text],{type:format==='csv'?'text/csv;charset=utf-8':'application/json'}));const a=document.createElement('a');a.href=url;a.download=`AionPartyMeter.${format}`;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);
   }
   panel=null;renderPanel();
  }else if(element.dataset.action)await action(element.dataset.action);
 };
 void run().catch(e=>toast(String(e),true));
});
root.addEventListener('change',event=>{
 const element=event.target as HTMLInputElement;
 if(element.id==='display-layout'||element.id==='layout-option')void chooseLayout(element.value).catch(e=>toast(String(e),true));
 if(element.id==='compact-option')void (element.checked?chooseLayout(layout):setCompact(false)).catch(e=>toast(String(e),true));
 if(element.id==='refresh-rate'){refreshRate=readRefreshRate(element.value);localStorage.setItem('apm-refresh-rate',String(refreshRate));}
 if(element.id==='motion-option'){motion=element.checked;localStorage.setItem('apm-motion',String(motion));render();}
 if(element.id==='target')void invoke('select_target',{target:element.value?Number(element.value):null}).then(()=>refresh(true)).catch(e=>toast(String(e),true));
 if(element.id==='file'){
  const file=element.files?.[0];if(!file)return;
  void(async()=>{
   if(file.size>64*1024*1024)throw new Error('Fichier trop volumineux (64 Mo maximum).');
   const text=await file.text();
   try{snapshot=validateSnapshot(JSON.parse(text));imported=true;viewedId=null;panel=null;selectedMember=null;render();toast('Rapport local importé.');}
   catch(error){
    if(file.name.endsWith('.json'))throw error;
    if(!native)throw new Error('Ouvre la capture JSONL avec l’application Windows pour la décoder.');
    await invoke('replay_text',{text});viewedId=null;imported=false;panel=null;await refresh();
   }
  })().catch(e=>toast(String(e),true)).finally(()=>{element.value='';});
 }
});
root.addEventListener('input',event=>{const input=event.target as HTMLInputElement;if(input.id==='opacity')document.documentElement.style.setProperty('--opacity',String(Number(input.value)/100));});
document.addEventListener('keydown',event=>{if(event.key==='Enter'&&(event.target as HTMLElement).id==='party-quick-name'){event.preventDefault();void action('quick-party').catch(e=>toast(String(e),true));}if(event.key==='Escape'){selectedMember=null;panel=null;renderPanel();}});
async function refresh(force=false):Promise<void>{
 if(!native)return;
 if(refreshing){await refreshing;if(!force)return;return refresh();}
 refreshing=(async()=>{
  const detailMember=!viewedId&&!imported?selectedMember:null;
  const next=await invoke<Dashboard>('dashboard',{detailMember,overview:overview&&!compact});
  dashboard=next;activity.observe(next.diagnostics,performance.now());
  if(!viewedId&&!imported)snapshot=dashboard.snapshot;
  render();
 })();
 try{await refreshing;}finally{refreshing=null;}
}

if(native){
 installWindowGestures();
 void(async()=>{for(;;){const start=performance.now();try{await refresh();}catch(e){toast(`Application : ${e}`,true);}await new Promise(resolve=>setTimeout(resolve,pollDelay(pollInterval(refreshRate,document.hidden,selectedMember!==null),performance.now()-start)));}})();
}
render();

// Coalesce pointer updates and retain the final position even when IPC completes
// after a fast release. Coordinates are converted to physical pixels for DPI.
function installWindowGestures():void {
 type Geometry={x:number;y:number;width:number;height:number;scale:number};
 type Gesture={mode:string;x:number;y:number;latestX:number;latestY:number;appliedX:number;appliedY:number;geometry:Geometry|null;running:boolean;scheduled:boolean;done:boolean};
 let gesture:Gesture|null=null;
 const apply=(g:Gesture):void=>{
  if(!g.geometry||g.running)return;
  if(g.appliedX===g.latestX&&g.appliedY===g.latestY){if(g.done&&gesture===g)gesture=null;return;}
  const geometry=transformGeometry(g.geometry,g.mode,g.latestX-g.x,g.latestY-g.y,compact);
  g.running=true;g.appliedX=g.latestX;g.appliedY=g.latestY;
  void invoke('window_transform',{geometry,resize:g.mode!=='Move'}).catch(e=>{gesture=null;toast(String(e),true);g.done=true;}).finally(()=>{g.running=false;if(gesture===g)schedule(g);});
 };
 const schedule=(g:Gesture):void=>{if(g.scheduled||g.running)return;g.scheduled=true;requestAnimationFrame(()=>{g.scheduled=false;if(gesture===g)apply(g);});};
 const attach=(element:HTMLElement,mode:string):void=>{
  element.addEventListener('pointerdown',event=>{
   if(event.button!==0||(mode==='Move'&&(event.target as HTMLElement).closest('nav')))return;
   event.preventDefault();element.setPointerCapture(event.pointerId);
   const g:Gesture={mode,x:event.screenX,y:event.screenY,latestX:event.screenX,latestY:event.screenY,appliedX:event.screenX,appliedY:event.screenY,geometry:null,running:false,scheduled:false,done:false};
   gesture=g;void invoke<Geometry>('window_geometry').then(value=>{g.geometry=value;schedule(g);}).catch(e=>{gesture=null;toast(String(e),true);});
  });
  element.addEventListener('pointermove',event=>{if(gesture){gesture.latestX=event.screenX;gesture.latestY=event.screenY;schedule(gesture);}});
  const finish=(event:PointerEvent):void=>{if(gesture){gesture.latestX=event.screenX;gesture.latestY=event.screenY;gesture.done=true;schedule(gesture);}if(element.hasPointerCapture(event.pointerId))element.releasePointerCapture(event.pointerId);};
  element.addEventListener('pointerup',finish);element.addEventListener('pointercancel',finish);
 };
 attach(root.querySelector<HTMLElement>('header')!,'Move');
 root.querySelectorAll<HTMLElement>('[data-resize]').forEach(grip=>attach(grip,grip.dataset.resize!));
}
