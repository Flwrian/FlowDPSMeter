import type {PartyMemberStatus} from './types';
export function partyStatus(members:PartyMemberStatus[],manual:boolean,rosterUpdates:number):string {
 const known=members.filter(r=>r.identityConfirmed).length,count=members.length;
 if(!count)return 'Personnage en attente';
 if(count===1&&members[0].member.local)return manual?'Solo':rosterUpdates?'Solo · liste reçue':'Vous seul suivi';
 return `${manual?'Groupe sélectionné':'Groupe reçu'} · ${known}/${count} identifiés`;
}
