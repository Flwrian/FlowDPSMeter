import {readFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import ts from 'typescript';
const uri=text=>'data:text/javascript;base64,'+Buffer.from(text).toString('base64');
const compile=name=>ts.transpileModule(readFileSync(new URL('../src/'+name,import.meta.url),'utf8'),{compilerOptions:{module:ts.ModuleKind.ES2022}}).outputText;
const hp=await import(uri(compile('target-health.ts').replace("'./view-dom'",JSON.stringify(uri('export const patchText=()=>{};')))));
const types=await import(uri(compile('types.ts')));
test('HP follows selected target or latest party target, never a sum of enemies',()=>{
 const targets=[{id:1,name:'Boss',currentHp:450,maxHp:1000},{id:2,name:'Add',currentHp:80,maxHp:100}];
 assert.equal(hp.healthTarget({targets,selectedTarget:null,trackedTarget:1}),targets[0]);
 assert.equal(hp.healthTarget({targets,selectedTarget:2,trackedTarget:1}),targets[1]);
 assert.equal(hp.healthTarget({targets,selectedTarget:null}),undefined);
});
test('unknown maximum or inconsistent readings never give a fictional percentage; dead is zero',()=>{
 assert.equal(hp.healthRatio({currentHp:50}),null);
 assert.equal(hp.healthRatio({maxHp:100}),null);
 assert.equal(hp.healthRatio({currentHp:110,maxHp:100}),null);
 assert.equal(hp.healthRatio({currentHp:0,maxHp:100}),0);
 assert.equal(hp.healthRatio({currentHp:50,maxHp:100}),.5);
});
test('old history remains readable, with no invented health or auto target',()=>{
 const old={...types.emptySnapshot(),targets:[{id:1,name:'Boss'}]};
 assert.equal(types.validateSnapshot(old),old);
 assert.equal(hp.healthTarget(old),undefined);
});
test('import rejects negative, infinite or zero max health but permits partial readings',()=>{
 for(const value of [{currentHp:-1},{maxHp:0},{maxHp:Infinity}])assert.throws(()=>types.validateSnapshot({...types.emptySnapshot(),targets:[{id:1,name:'Boss',...value}]}));
 assert.doesNotThrow(()=>types.validateSnapshot({...types.emptySnapshot(),targets:[{id:1,name:'Boss',currentHp:0,maxHp:null}]}));
});
