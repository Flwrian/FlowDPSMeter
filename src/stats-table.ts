import type { Row,Totals } from './types';
import { patchText } from './view-dom';

export const columns=[['name','Membre'],['class','Classe'],['damage','Dégâts'],['dps','DPS'],['contribution','Part'],['healing','Soins'],['hps','HPS'],['healingReceived','Soins reçus'],['damageReceived','Dégâts reçus'],['deaths','Morts']] as const;
export type Column=typeof columns[number][0];
const numbers=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:0});
const percent=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:1});
const collator=new Intl.Collator('fr-FR',{numeric:true,sensitivity:'base'});
export function sortStats(rows:Row[],key:Column,descending:boolean):Row[] {
 return [...rows].sort((a,b)=>{
  if(key!=='name'&&key!=='class'&&a.identityConfirmed!==b.identityConfirmed){
   if(a.identityConfirmed===false)return 1;if(b.identityConfirmed===false)return -1;
  }
  const difference=key==='name'||key==='class'?collator.compare(key==='name'?a.member.name:a.member.class,key==='name'?b.member.name:b.member.class)
   :key==='contribution'?a.contribution-b.contribution:a.totals[key]-b.totals[key];
  return (descending?-difference:difference)||collator.compare(a.member.name,b.member.name);
 });
}

export function renderStatsTable(container:HTMLElement,rows:Row[],totals:Totals,key:Column,descending:boolean):void {
 const focused=document.activeElement;
 if(!container.querySelector('table')){
  container.innerHTML=`<table class="all-stats"><caption>Statistiques observées du groupe · clique un membre pour ses détails</caption><thead><tr>${columns.map(([id,label])=>`<th scope="col"><button data-column="${id}">${label}</button></th>`).join('')}</tr></thead><tbody></tbody><tfoot><tr>${columns.map(()=>'<td></td>').join('')}</tr></tfoot></table>`;
 }
 for(const [id,label]of columns){
  const button=container.querySelector<HTMLButtonElement>(`[data-column="${id}"]`)!;
  button.parentElement!.setAttribute('aria-sort',key===id?(descending?'descending':'ascending'):'none');
  patchText(button,label+(key===id?(descending?' ↓':' ↑'):''));
 }
 const body=container.querySelector('tbody')!;
 const existing=new Map(Array.from(body.querySelectorAll<HTMLTableRowElement>('tr')).map(node=>[node.dataset.key!,node]));
 let previous:Element|null=null;
 for(const row of sortStats(rows,key,descending)){
  let node=existing.get(row.member.key);
  if(!node){
   node=document.createElement('tr');node.dataset.key=row.member.key;
   node.innerHTML='<th scope="row"><button data-member=""><strong></strong><small></small></button></th>'+columns.slice(1).map(()=>'<td></td>').join('');
   node.querySelector<HTMLButtonElement>('button')!.dataset.member=row.member.key;
  }
  node.classList.toggle('departed',!row.member.present);
  patchText(node.querySelector('strong'),row.member.name+(row.member.local?' · vous':''));
  patchText(node.querySelector('small'),row.identityConfirmed===false?'Identité en attente':row.member.present?'':'A quitté le groupe');
  columns.slice(1).forEach(([id],i)=>{
   const value=id==='name'?row.member.name:id==='class'?row.member.class||'Inconnue':row.identityConfirmed===false?'—':id==='contribution'?percent.format(row.contribution)+' %':numbers.format(row.totals[id]);
   patchText(node!.children[i+1],value);
  });
  if((previous?previous.nextElementSibling:body.firstElementChild)!==node)body.insertBefore(node,previous?previous.nextElementSibling:body.firstElementChild);
  previous=node;existing.delete(row.member.key);
 }
 for(const node of existing.values())node.remove();
 const foot=container.querySelector('tfoot tr')!;
 for(const [i,[id]]of columns.entries())patchText(foot.children[i],id==='name'?'Total observé':id==='class'?(rows.some(r=>r.identityConfirmed===false)?'Partiel':'Groupe'):id==='contribution'?(totals.damage?'100 %':'0 %'):numbers.format(totals[id]));
 if(focused instanceof HTMLButtonElement&&container.contains(focused)&&document.activeElement!==focused)focused.focus({preventScroll:true});
}
