import type {Snapshot,Target} from './types';
import {patchText} from './view-dom';

const exact=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:0});
export function healthTarget(snapshot:Snapshot):Target|undefined {
 const id=snapshot.selectedTarget??snapshot.trackedTarget;
 return id==null?undefined:snapshot.targets.find(target=>target.id===id);
}
export function healthRatio(target:Target|undefined):number|null {
 if(target?.currentHp==null||target.maxHp==null||target.maxHp<=0||target.currentHp>target.maxHp)return null;
 return target.currentHp/target.maxHp;
}
export function renderTargetHealth(container:HTMLElement,snapshot:Snapshot):void {
 if(!container.querySelector('.target-hp-value'))container.innerHTML='<span class="target-hp-name"></span><span class="target-hp-value"></span><span class="target-hp-percent"></span><span class="target-hp-track" aria-hidden="true"><i></i></span>';
 const target=healthTarget(snapshot),ratio=healthRatio(target);
 const value=(hp:number|null|undefined)=>hp==null?'—':exact.format(hp);
 patchText(container.querySelector('.target-hp-name'),target?.name??'PV cible');
 patchText(container.querySelector('.target-hp-value'),`${value(target?.currentHp)} / ${value(target?.maxHp)} PV`);
 patchText(container.querySelector('.target-hp-percent'),ratio==null?'':`${Math.round(ratio*100)} %`);
 container.dataset.known=String(ratio!==null);
 container.style.setProperty('--target-hp',`${(ratio??0)*100}%`);
 container.title=`${target?.name??'Cible non identifiée'} · PV actuels / maximum : ${value(target?.currentHp)} / ${value(target?.maxHp)}. Dernières valeurs reçues du jeu.${ratio===null?' Données manquantes ou incohérentes : pourcentage indisponible.':''}${snapshot.selectedTarget==null?' Suivi de la dernière cible attaquée par le groupe.':''}`;
 container.setAttribute('aria-label',container.title);
}
