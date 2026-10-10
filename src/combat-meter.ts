import type {Metric,Row,Snapshot} from './types';
import {patchText} from './view-dom';

const compact=new Intl.NumberFormat('fr-FR',{notation:'compact',maximumFractionDigits:1});
const exact=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:0});
const percent=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:1});
const labels:Record<Metric,string>={damage:'Dégâts',healing:'Soins',damageReceived:'Reçus'};
const rateLabels:Record<Metric,string>={damage:'DPS',healing:'HPS',damageReceived:'Reçus / s'};
const kinds=['damage','healing','damageReceived'] as const;
const allLabels:Record<Metric,string>={damage:'Dégâts',healing:'Soins',damageReceived:'Tank'};
const optionKeys=new WeakMap<HTMLSelectElement,string>();
export const meterNumber=(value:number):string=>compact.format(value).replace(/[\u00a0\u202f]/g,'');

// Original geometric class symbols; no artwork from the reference application.
const shapes:Record<string,[string,string]>={
 Gladiateur:['#d9a066','M4 4l12 12M13 13l3-3M3 7l4-4M3 3l3 1-2 2z'],
 Templier:['#79afc9','M10 2l6 3v5c0 4-6 8-6 8s-6-4-6-8V5zM10 6v7M7 9h6'],
 'Rôdeur':['#79c99b','M5 3c9 3 9 11 0 14M5 3v14M3 10h14M14 7l3 3-3 3'],
 Assassin:['#b080ce','M6 3l2 9-3 4M14 3l-2 9 3 4M4 13l4 2M12 15l4-2'],
 Sorcier:['#a88ae3','M10 2l2 5 5 3-5 2-2 6-2-6-5-2 5-3zM4 3l2 2M15 15l2 2'],
 Clerc:['#d2bb68','M7 3h6v4h4v6h-4v4H7v-4H3V7h4z'],
 Spiritualiste:['#70bbc6','M10 2c-2 5 5 5 3 9 4-1 4 6-3 7-8-1-7-8-3-10 0 3 2 2 3-6z'],
 'Aède':['#99c3ab','M7 3v10M7 6l7-2v8M3 14c0-2 4-3 4-1s-4 3-4 1M10 13c0-2 4-3 4-1s-4 3-4 1'],
 Combattant:['#d78b85','M4 8l3-3 3 3 3-3 3 3-2 7H6zM7 8v4M10 8v4M13 8v4'],
};
export function classVisual(name:string):{color:string;symbol:string} {
 const [color,path]=shapes[name]??['#8d9ba9','M10 3l7 7-7 7-7-7zM10 7v6'];
 return {color,symbol:`<svg viewBox="0 0 20 20" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"><path d="${path}"/></svg>`};
}
export function combatValues(row:Row,snapshot:Snapshot,kind:Metric):{known:boolean;total:number;rate:number;share:number} {
 const known=row.identityConfirmed!==false;
 const total=known?row.totals[kind]:0;
 const rate=kind==='damage'?row.totals.dps:kind==='healing'?row.totals.hps:row.totals.damageReceived/Math.max(1,snapshot.duration);
 return {known,total,rate:known?rate:0,share:known&&snapshot.totals[kind]>0?total/snapshot.totals[kind]*100:0};
}

function combinedCell(element:HTMLElement,total:number,rate:number,known:boolean,metric:Metric,share?:number):void {
 patchText(element.querySelector('strong'),known?meterNumber(total):'—');
 patchText(element.querySelector('small'),known?`${meterNumber(rate)} /s`:'—');
 element.title=known?`${labels[metric]} : ${exact.format(total)} au total · ${exact.format(rate)} / s${share===undefined?'':` · ${percent.format(share)} % du groupe`}${metric==='damageReceived'?'\nTank = dégâts reçus observés.':''}`:'Identité en attente : valeur inconnue';
}
export function renderCombat(container:HTMLElement,snapshot:Snapshot,kind:Metric,motion:boolean,rows:Row[],all=false):void {
 if(!container.querySelector('.combat-encounter'))container.innerHTML=`<div class="combat-encounter"><div class="combat-target-line"><span class="combat-target-icon" aria-hidden="true">◇</span><select id="combat-target" aria-label="Filtrer les dégâts par cible"></select><span class="combat-group-total" title="Total observé pour la statistique sélectionnée"></span></div><div class="combat-metric-line"><div class="combat-metrics" role="tablist" aria-label="Statistique du classement">${(['damage','healing','damageReceived'] as const).map(metric=>`<button role="tab" data-compact-sort="${metric}">${labels[metric]}</button>`).join('')}</div><span class="combat-group-rate"><small></small><strong></strong></span></div></div><div class="combat-columns"><span>MEMBRE · TOTAL</span><span class="combat-rate-label"></span><span>PART</span></div><div class="combat-list"></div><div class="combat-empty" hidden><strong>En attente du groupe</strong><span>Démarre la capture pour identifier les membres.</span><button data-action="party">Ouvrir Groupe</button></div>`;
 container.dataset.metric=kind;container.classList.toggle('combat-motion',motion);
 if(all&&!container.querySelector('.combat-all-head')){
  container.querySelector('.combat-encounter')!.insertAdjacentHTML('beforeend',`<div class="combat-all-summary"><span>Groupe</span>${kinds.map(metric=>`<span class="combat-stat ${metric}"><strong></strong><small></small></span>`).join('')}</div>`);
  container.querySelector('.combat-columns')!.insertAdjacentHTML('afterend',`<div class="combat-all-head"><span>Membre</span>${kinds.map(metric=>`<button data-compact-sort="${metric}" aria-pressed="false" title="Trier par ${metric==='damageReceived'?'Tank : dégâts reçus':allLabels[metric]}">${allLabels[metric]}</button>`).join('')}</div>`);
 }
 const target=container.querySelector<HTMLSelectElement>('#combat-target')!;
 const key=JSON.stringify(snapshot.targets);
 if(optionKeys.get(target)!==key){target.replaceChildren(new Option('Toutes les cibles',''),...snapshot.targets.map(t=>new Option(t.name,String(t.id))));optionKeys.set(target,key);}
 target.value=snapshot.selectedTarget===null?'':String(snapshot.selectedTarget);
 target.title=all||kind==='damage'?'Filtrer les dégâts ; les soins et dégâts reçus restent ceux du groupe.':'Le filtre cible concerne uniquement les dégâts.';
 const total=container.querySelector<HTMLElement>('.combat-group-total')!;
 patchText(total,`${meterNumber(snapshot.totals[kind])} ${labels[kind].toLocaleLowerCase('fr')}`);
 total.title=`${exact.format(snapshot.totals[kind])} ${labels[kind].toLocaleLowerCase('fr')} comptés pour le groupe`;
 const groupRate=snapshot.totals[kind]/Math.max(1,snapshot.duration);
 patchText(container.querySelector('.combat-group-rate strong'),snapshot.startedAt!==null?meterNumber(groupRate):'—');
 patchText(container.querySelector('.combat-group-rate small'),`${rateLabels[kind]} groupe`);
 patchText(container.querySelector('.combat-rate-label'),rateLabels[kind]);
 for(const button of container.querySelectorAll<HTMLButtonElement>('[data-compact-sort]')){const selected=button.dataset.compactSort===kind;button.setAttribute(button.getAttribute('role')==='tab'?'aria-selected':'aria-pressed',String(selected));button.classList.toggle('selected',selected);}
 if(all)for(const metric of kinds)combinedCell(container.querySelector<HTMLElement>(`.combat-all-summary .${metric}`)!,snapshot.totals[metric],snapshot.totals[metric]/Math.max(1,snapshot.duration),snapshot.startedAt!==null,metric);
 const list=container.querySelector<HTMLElement>('.combat-list')!;
 container.querySelector<HTMLElement>('.combat-empty')!.hidden=rows.length>0;
 const old=new Map(Array.from(list.querySelectorAll<HTMLButtonElement>('.combat-row')).map(node=>[node.dataset.member!,node]));
 const focused=document.activeElement,max=Math.max(1,...rows.filter(r=>r.identityConfirmed!==false).map(r=>r.totals[kind]));
 let previous:Element|null=null;
 for(const row of rows){
  let node=old.get(row.member.key);
  if(!node){node=document.createElement('button');node.className='combat-row';node.dataset.member=row.member.key;node.innerHTML='<span class="combat-bar"></span><span class="combat-class"></span><span class="combat-identity"><strong></strong><small class="combat-me"></small><small class="combat-total"></small><small class="combat-pending"></small></span>'+(all?kinds.map(metric=>`<span class="combat-stat ${metric}"><strong></strong><small></small></span>`).join(''):'<strong class="combat-rate"></strong><span class="combat-share"></span>');}
  const value=combatValues(row,snapshot,kind),visual=classVisual(row.member.class);
  node.style.setProperty('--member-color',visual.color);node.style.setProperty('--combat-bar',`${Math.min(100,value.total/max*100)}%`);
  node.classList.toggle('departed',!row.member.present);node.classList.toggle('unconfirmed',!value.known);node.classList.toggle('local-member',row.member.local);
  const symbol=node.querySelector<HTMLElement>('.combat-class')!;
  if(symbol.dataset.job!==row.member.class){symbol.innerHTML=visual.symbol;symbol.dataset.job=row.member.class;}symbol.title=row.member.class||'Classe inconnue';
  patchText(node.querySelector('.combat-identity strong'),row.member.name);
  patchText(node.querySelector('.combat-me'),row.member.local?'VOUS':'');
  patchText(node.querySelector('.combat-total'),value.known?meterNumber(value.total):'—');
  patchText(node.querySelector('.combat-pending'),!value.known?'?':!row.member.present?'↪':'');
  patchText(node.querySelector('.combat-rate'),value.known?meterNumber(value.rate):'—');
  patchText(node.querySelector('.combat-share'),value.known?`${percent.format(value.share)}%`:'—');
  if(all)for(const metric of kinds){const cell=combatValues(row,snapshot,metric);combinedCell(node.querySelector<HTMLElement>(`.combat-stat.${metric}`)!,cell.total,cell.rate,cell.known,metric,cell.share);}
  node.title=`${row.member.name} · ${row.member.class||'Classe inconnue'}${row.member.local?' · vous':''}${!row.member.present?' · a quitté le groupe':''}\n${value.known?`${exact.format(value.total)} ${labels[kind].toLocaleLowerCase('fr')} · ${exact.format(value.rate)} / s · ${percent.format(value.share)} %`:'Identité en attente : statistiques inconnues'}\nOuvrir les compétences et graphiques`;
  node.setAttribute('aria-label',`${labels[kind]} · détails de ${row.member.name}${!value.known?' · identité en attente':''}`);
  if(all){node.title=`${row.member.name} · ${row.member.class||'Classe inconnue'}${!row.member.present?' · ancien membre':''}${row.member.local?' · vous':''}\n${kinds.map(metric=>node!.querySelector<HTMLElement>(`.combat-stat.${metric}`)!.title).join('\n')}\nOuvrir les compétences et graphiques`;node.setAttribute('aria-label',`Dégâts, soins et tank · détails de ${row.member.name}${!value.known?' · identité en attente':''}`);}
  if((previous?previous.nextElementSibling:list.firstElementChild)!==node)list.insertBefore(node,previous?previous.nextElementSibling:list.firstElementChild);
  previous=node;old.delete(row.member.key);
 }
 for(const node of old.values())node.remove();
 if(focused instanceof HTMLButtonElement&&container.contains(focused)&&document.activeElement!==focused)focused.focus({preventScroll:true});
}
