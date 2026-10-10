import type {Metric,Snapshot,Row} from './types';
import {patchText} from './view-dom';
import {meterTotal} from './combat-meter';

const format=new Intl.NumberFormat('fr-FR',{notation:'compact',maximumFractionDigits:1});
const exact=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:0});
const labels:Record<Metric,string>={damage:'Dégâts',healing:'Soins',damageReceived:'Reçus'};
const amounts=new WeakMap<Element,{value:number;combat:string}>();
const animations=new WeakMap<Element,Animation>();
const kinds=['damage','healing','damageReceived'] as const;

export function metricCell(element:HTMLElement,amount:number,rate:number,known:boolean,combat:string,motion:boolean):void {
 const value=element.querySelector('strong')!;
 const previous=amounts.get(value);
 patchText(value,known?meterTotal(amount):'—');patchText(element.querySelector('small'),known?`${format.format(rate)} / s`:'En attente');
 element.title=known?`${exact.format(amount)} au total · ${exact.format(rate)} / s`:'Identité non reçue : valeur inconnue';
 if(motion&&known&&previous?.combat===combat&&amount>previous.value){
  animations.get(value)?.cancel();animations.set(value,value.animate([{backgroundColor:'#ffffff18'},{backgroundColor:'transparent'}],{duration:180,easing:'ease-out'}));
 }
 amounts.set(value,{value:amount,combat});
}
export function renderCompact(container:HTMLElement,snapshot:Snapshot,metric:Metric,motion:boolean):void {
 if(!container.querySelector('.mini-head')){
  container.innerHTML=`<div class="mini-head"><span>Membre</span>${kinds.map(kind=>`<button data-compact-sort="${kind}" title="Trier par ${labels[kind].toLowerCase()}">${labels[kind]}</button>`).join('')}</div><div class="mini-body"></div><div class="mini-total"><span>Total observé</span>${kinds.map(kind=>`<div class="mini-cell ${kind}"><strong></strong><small></small></div>`).join('')}</div>`;
 }
 for(const kind of kinds){const head=container.querySelector<HTMLButtonElement>(`[data-compact-sort="${kind}"]`)!;patchText(head,labels[kind]+(kind===metric?' ↓':''));head.setAttribute('aria-pressed',String(kind===metric));}
 const body=container.querySelector<HTMLElement>('.mini-body')!;
 const old=new Map(Array.from(body.querySelectorAll<HTMLButtonElement>('.mini-row')).map(node=>[node.dataset.member!,node]));
 const rows=[...snapshot.members].sort((a,b)=>{
  if(a.identityConfirmed===false&&b.identityConfirmed!==false)return 1;if(b.identityConfirmed===false&&a.identityConfirmed!==false)return -1;
  return b.totals[metric]-a.totals[metric]||a.member.name.localeCompare(b.member.name,'fr');
 });
 const max=Math.max(1,...rows.map(r=>r.totals[metric]));
 const focused=document.activeElement;
 let previous:Element|null=null;
 for(const row of rows){
  let node=old.get(row.member.key);
  if(!node){node=document.createElement('button');node.className='mini-row';node.dataset.member=row.member.key;
   node.innerHTML=`<span class="mini-name"><strong></strong><small></small></span>${kinds.map(kind=>`<span class="mini-cell ${kind}"><strong></strong><small></small></span>`).join('')}`;}
  node.classList.toggle('departed',!row.member.present);node.classList.toggle('unconfirmed',row.identityConfirmed===false);
  node.style.setProperty('--mini-bar',`${row.totals[metric]/max*100}%`);
  patchText(node.querySelector('.mini-name strong'),row.member.name);
  patchText(node.querySelector('.mini-name small'),row.identityConfirmed===false?'Identité en attente':!row.member.present?'Ancien membre':(row.member.class||'Membre')+(row.member.local?' · vous':''));
  node.title=`${row.member.name} · ${row.identityConfirmed===false?'Identité en attente':row.member.class||'Classe inconnue'}${!row.member.present?' · Ancien membre':''}${row.member.local?' · vous':''}`;
  node.setAttribute('aria-label',`Détails de ${row.member.name}`);
  for(const kind of kinds)metricCell(node.querySelector<HTMLElement>(`.${kind}`)!,row.totals[kind],memberRate(row,kind,snapshot.duration),row.identityConfirmed!==false,snapshot.id,motion);
  if((previous?previous.nextElementSibling:body.firstElementChild)!==node)body.insertBefore(node,previous?previous.nextElementSibling:body.firstElementChild);
  previous=node;old.delete(row.member.key);
 }
 for(const node of old.values())node.remove();
 const total=container.querySelector<HTMLElement>('.mini-total')!;
 for(const kind of kinds)metricCell(total.querySelector<HTMLElement>(`.${kind}`)!,snapshot.totals[kind],snapshot.totals[kind]/Math.max(1,snapshot.duration),snapshot.startedAt!==null,snapshot.id,motion);
 if(focused instanceof HTMLButtonElement&&container.contains(focused)&&document.activeElement!==focused)focused.focus({preventScroll:true});
}
export function memberRate(row:Row,kind:Metric,span:number):number {return kind==='damage'?row.totals.dps:kind==='healing'?row.totals.hps:row.totals.damageReceived/Math.max(1,span);}
