import {readFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import ts from 'typescript';
const uri=text=>'data:text/javascript;base64,'+Buffer.from(text).toString('base64');
const compile=name=>ts.transpileModule(readFileSync(new URL('../src/'+name,import.meta.url),'utf8'),{compilerOptions:{module:ts.ModuleKind.ES2022,target:ts.ScriptTarget.ES2020}}).outputText;
const feedback=await import(uri(compile('capture-feedback.ts')));
const layout=await import(uri(compile('visual-layout.ts').replace("'./compact-meter'",JSON.stringify(uri('export const renderCompact=()=>{};export const metricCell=()=>{};export const memberRate=()=>0;'))).replace("'./view-dom'",JSON.stringify(uri('export const patchText=()=>{};')))));
const diag=(patch={})=>({mode:'live',npcap:true,connection:'flow1',packets:100,losses:0,error:null,...patch});
const snapshot=(patch={})=>({active:false,startedAt:null,warnings:[],...patch});
const members=[{member:{name:'Me',local:true},identityConfirmed:true},{member:{name:'Healer'},identityConfirmed:false}];
test('packet activity requires an observed increase; stale traffic does not fake a disconnection',()=>{
 const tracker=new feedback.ActivityTracker();tracker.observe(diag(),1000);assert.equal(tracker.age(1500),null);
 tracker.observe(diag({packets:101}),1600);assert.equal(tracker.age(1800),.2);
 const state=feedback.captureFeedback(diag({packets:101}),snapshot(),members,1,tracker.age(5000),false);
 assert.equal(state.capture,'Capture active');assert.equal(state.activity,false);assert.match(state.age,/3 s/);
});
test('activity resets on flow change, replay, stop and counter reset',()=>{
 for(const patch of [{connection:'flow2'},{mode:'replay'},{mode:'stopped'},{packets:0}]){
  const tracker=new feedback.ActivityTracker();tracker.observe(diag(),0);tracker.observe(diag({packets:110}),100);tracker.observe(diag(patch),200);assert.equal(tracker.age(300),null);
 }
});
test('group feedback uses actual confirmations and names without changing membership',()=>{
 const before=JSON.stringify(members);const state=feedback.captureFeedback(diag(),snapshot(),members,4,null,false);
 assert.match(state.groupTitle,/1\/2 identités confirmées/);assert.match(state.groupTitle,/Healer : identité en attente/);assert.match(state.groupTitle,/4/);assert.equal(JSON.stringify(members),before);
 assert.equal(state.partial,false);
});
test('partial data requires real losses or a recorded partial warning',()=>{
 assert.equal(feedback.captureFeedback(diag({losses:1}),snapshot(),members,0,null,false).partial,true);
 assert.equal(feedback.captureFeedback(diag(),snapshot({warnings:['Invocation inconnue']}),members,0,null,false).partial,false);
 assert.equal(feedback.captureFeedback(diag(),snapshot({warnings:['Capture partielle : événements perdus']}),members,0,null,true).partial,true);
});
test('historical report isolates live losses and packet pulse; capture can continue in background',()=>{
 const state=feedback.captureFeedback(diag({losses:9}),snapshot({startedAt:123}),members,1,.1,true);
 assert.equal(state.capture,'Rapport local');assert.equal(state.activity,false);assert.equal(state.partial,false);assert.equal(state.combat,'Combat terminé');assert.match(state.title,/arrière-plan/);
});
test('capture search, errors, replay and combat states are distinguishable',()=>{
 assert.equal(feedback.captureFeedback(diag({npcap:false,mode:'stopped'}),snapshot(),[],0,null,false).capture,'Npcap absent');
 assert.equal(feedback.captureFeedback(diag({mode:'stopped'}),snapshot(),[],0,null,false).capture,'Capture arrêtée');
 assert.equal(feedback.captureFeedback(diag({connection:null}),snapshot(),[],0,null,false).capture,'Recherche du jeu');
 assert.equal(feedback.captureFeedback(diag({error:'Accès refusé'}),snapshot(),[],0,null,false).tone,'error');
 assert.equal(feedback.captureFeedback(diag({mode:'replay'}),snapshot({active:true,startedAt:1}),[],0,.1,false).capture,'Replay local');
 assert.equal(feedback.captureFeedback(null,snapshot(),[],0,null,false).capture,'Visionneuse');
 assert.equal(feedback.captureFeedback(diag(),snapshot({active:true}),[],0,null,false).combat,'Combat en cours');
});
test('layout preference validates stored values; sizes are bounded for large groups',()=>{
 assert.equal(layout.readLayout('cards'),'cards');assert.equal(layout.readLayout('panels'),'panels');assert.equal(layout.readLayout('broken'),'band');
 assert.deepEqual(layout.layoutSize('band',4),{width:400,height:256});assert.deepEqual(layout.layoutSize('cards',4),{width:460,height:324});assert.deepEqual(layout.layoutSize('panels',4),{width:780,height:344});
 assert.equal(layout.layoutSize('cards',100).height,620);assert.equal(layout.layoutSize('band',0).height,200);
});
test('panels independently sort all members including healer and zero damage; input is preserved',()=>{
 const rows=[{member:{name:'Tank'},totals:{damage:10,healing:0,damageReceived:50}},{member:{name:'Healer'},totals:{damage:0,healing:60,damageReceived:0}},{member:{name:'DPS'},totals:{damage:100,healing:0,damageReceived:10}},{member:{name:'Pending'},identityConfirmed:false,totals:{damage:999,healing:999,damageReceived:999}}];
 const before=JSON.stringify(rows);
 assert.equal(layout.metricRows(rows,'damage')[0].member.name,'DPS');assert.equal(layout.metricRows(rows,'healing')[0].member.name,'Healer');assert.equal(layout.metricRows(rows,'damageReceived')[0].member.name,'Tank');assert.equal(layout.metricRows(rows,'damage').at(-1).member.name,'Pending');assert.equal(JSON.stringify(rows),before);
});
