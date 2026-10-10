import type {Metric,Row,Snapshot} from './types';
import {renderCompact,metricCell,memberRate} from './compact-meter';
import {patchText} from './view-dom';
import {renderCombat} from './combat-meter';

export type Layout='combat'|'combat-all'|'band'|'cards'|'panels';
export const layouts=[['combat','Combat'],['combat-all','Combat +'],['band','Bandeau'],['cards','Cartes'],['panels','3 panneaux']] as const;
export function readLayout(value:string|null):Layout {return value==='cards'||value==='panels'||value==='band'||value==='combat-all'?value:'combat';}
export function adoptCombat(value:string|null,applied:boolean):boolean {return !applied&&value!=='cards'&&value!=='panels'&&value!=='combat-all';}
export function layoutSize(layout:Layout,members:number):{width:number;height:number} {
 const count=Math.max(0,members);
 return layout==='combat'?{width:360,height:Math.min(560,Math.max(280,184+count*27))}
  :layout==='combat-all'?{width:400,height:Math.min(560,Math.max(330,208+count*33))}
  :layout==='cards'?{width:460,height:Math.min(620,Math.max(260,160+Math.ceil(count/2)*82))}
  :layout==='panels'?{width:780,height:Math.min(620,Math.max(260,200+count*36))}
  :{width:400,height:Math.min(450,Math.max(200,160+count*24))};
}
export function metricRows(rows:Row[],kind:Metric):Row[] {
 return [...rows].sort((a,b)=>{
  if(a.identityConfirmed===false&&b.identityConfirmed!==false)return 1;
  if(b.identityConfirmed===false&&a.identityConfirmed!==false)return -1;
  return b.totals[kind]-a.totals[kind]||a.member.name.localeCompare(b.member.name,'fr');
 });
}
export function renderVisualLayout(container:HTMLElement,snapshot:Snapshot,metric:Metric,motion:boolean,layout:Layout):void {
 const structure=layout==='combat'||layout==='combat-all'?layout:layout==='panels'?'panels':'compact';
 if(container.dataset.structure!==structure){container.replaceChildren();container.dataset.structure=structure;}
 container.dataset.layout=layout;
 if(layout==='combat'||layout==='combat-all'){renderCombat(container,snapshot,metric,motion,metricRows(snapshot.members,metric),layout==='combat-all');return;}
 if(layout!=='panels'){
  renderCompact(container,snapshot,metric,motion);
  patchText(container.querySelector('.mini-head>span'),layout==='cards'?'Trier':'Membre');
  return;
 }
 const kinds=['damage','healing','damageReceived'] as const;
 const labels={damage:'Dégâts',healing:'Soins',damageReceived:'Dégâts reçus'};
 if(!container.querySelector('.meter-panels'))container.innerHTML=`<div class="meter-panels">${kinds.map(kind=>`<article class="meter-panel ${kind}" data-kind="${kind}"><div class="meter-panel-head"><h3>${labels[kind]}</h3><div class="mini-cell ${kind}"><strong></strong><small></small></div><span>Total observé du groupe</span></div><div class="panel-list" aria-label="${labels[kind]} par membre"></div></article>`).join('')}</div>`;
 const focused=document.activeElement;
 for(const kind of kinds){
  const column=container.querySelector<HTMLElement>(`[data-kind="${kind}"]`)!;
  metricCell(column.querySelector<HTMLElement>('.mini-cell')!,snapshot.totals[kind],snapshot.totals[kind]/Math.max(1,snapshot.duration),snapshot.startedAt!==null,snapshot.id,motion);
  const list=column.querySelector<HTMLElement>('.panel-list')!;
  const old=new Map(Array.from(list.querySelectorAll<HTMLButtonElement>('.panel-row')).map(node=>[node.dataset.member!,node]));
  const rows=metricRows(snapshot.members,kind),max=Math.max(1,...rows.map(r=>r.totals[kind]));
  let previous:Element|null=null;
  for(const row of rows){
   let node=old.get(row.member.key);
   if(!node){node=document.createElement('button');node.className='panel-row';node.dataset.member=row.member.key;node.dataset.panelMetric=kind;
    node.innerHTML=`<span class="panel-name"><strong></strong><small></small></span><span class="mini-cell ${kind}"><strong></strong><small></small></span>`;}
   node.classList.toggle('departed',!row.member.present);node.classList.toggle('unconfirmed',row.identityConfirmed===false);
   node.style.setProperty('--panel-bar',`${row.totals[kind]/max*100}%`);
   patchText(node.querySelector('.panel-name strong'),row.member.name+(row.member.local?' · vous':''));
   patchText(node.querySelector('.panel-name small'),row.identityConfirmed===false?'Identité en attente':!row.member.present?'Ancien membre':row.member.class||'Membre');
   node.setAttribute('aria-label',`${labels[kind]} · détails de ${row.member.name}`);
   metricCell(node.querySelector<HTMLElement>('.mini-cell')!,row.totals[kind],memberRate(row,kind,snapshot.duration),row.identityConfirmed!==false,snapshot.id,motion);
   if((previous?previous.nextElementSibling:list.firstElementChild)!==node)list.insertBefore(node,previous?previous.nextElementSibling:list.firstElementChild);
   previous=node;old.delete(row.member.key);
  }
  for(const node of old.values())node.remove();
 }
 if(focused instanceof HTMLButtonElement&&container.contains(focused)&&document.activeElement!==focused)focused.focus({preventScroll:true});
}
