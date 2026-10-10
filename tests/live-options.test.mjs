import {readFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import ts from 'typescript';
const compile=name=>'data:text/javascript;base64,'+Buffer.from(ts.transpileModule(readFileSync(new URL('../src/'+name,import.meta.url),'utf8'),{compilerOptions:{module:ts.ModuleKind.ES2022,target:ts.ScriptTarget.ES2020}}).outputText).toString('base64');
const {readRefreshRate,pollInterval,pollDelay}=await import(compile('live-options.ts'));
const {partyStatus}=await import(compile('party-status.ts'));
const member=(name,local,identityConfirmed)=>({member:{name,local},identityConfirmed});
test('default is 10 Hz; invalid stored rates fall back; only supported settings accepted',()=>{
 assert.equal(readRefreshRate(null),10);assert.equal(readRefreshRate('1000'),10);assert.equal(readRefreshRate('4'),4);assert.equal(readRefreshRate('20'),20);
});
test('details and hidden windows reduce work; no busy catch-up loop for slow IPC',()=>{
 assert.equal(pollInterval(10,false,false),100);assert.equal(pollInterval(20,false,false),50);
 assert.equal(pollInterval(20,false,true),250);assert.equal(pollInterval(20,true,false),1000);
 assert.equal(pollDelay(100,20),80);assert.equal(pollDelay(100,300),8);
});
test('auto without roster reports one followed player rather than claiming a solo party',()=>{
 assert.equal(partyStatus([member('Me',true,true)],false,0),'Vous seul suivi');
 assert.equal(partyStatus([member('Me',true,true)],true,0),'Solo');
 assert.equal(partyStatus([],false,0),'Personnage en attente');
});
test('pending manual identity shows confirmed count without treating it as a zero player',()=>{
 assert.equal(partyStatus([member('Me',true,true),member('Ally',false,false)],true,0),'Groupe sélectionné · 1/2 identifiés');
 assert.equal(partyStatus([member('Me',true,true),member('Ally',false,true)],false,1),'Groupe reçu · 2/2 identifiés');
});
