import { readFileSync } from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import ts from 'typescript';
const compiled=ts.transpileModule(readFileSync(new URL('../src/window-geometry.ts',import.meta.url),'utf8'),{compilerOptions:{module:ts.ModuleKind.ES2022}}).outputText;
const {transformGeometry}=await import('data:text/javascript;base64,'+Buffer.from(compiled).toString('base64'));
const origin={x:-400,y:100,width:800,height:700,scale:1.25};
test('moving scales CSS coordinates to physical pixels on a secondary monitor',()=>{
 assert.deepEqual(transformGeometry(origin,'Move',80,40),{...origin,x:-300,y:150});
 assert.deepEqual(origin,{x:-400,y:100,width:800,height:700,scale:1.25});
});
test('south-east resize preserves the top-left corner',()=>{
 assert.deepEqual(transformGeometry(origin,'SouthEast',40,80),{...origin,width:850,height:800});
});
test('north-west resize preserves the opposite edges',()=>{
 const result=transformGeometry(origin,'NorthWest',40,80);
 assert.equal(result.x+result.width,origin.x+origin.width);
 assert.equal(result.y+result.height,origin.y+origin.height);
 assert.equal(result.width,750);assert.equal(result.height,600);
});
test('shrinking clamps to the logical minimum after scaling',()=>{
 const result=transformGeometry(origin,'SouthEast',-1000,-1000);
 assert.equal(result.width,525);assert.equal(result.height,525);
});
test('minimum size still preserves the opposite north-west edges',()=>{
 const result=transformGeometry(origin,'NorthWest',1000,1000);
 assert.equal(result.width,525);assert.equal(result.height,525);
 assert.equal(result.x+result.width,origin.x+origin.width);assert.equal(result.y+result.height,origin.y+origin.height);
});
test('compact overlay can shrink to 320 by 180 logical pixels at high DPI',()=>{
 const result=transformGeometry(origin,'NorthWest',1000,1000,true);
 assert.equal(result.width,400);assert.equal(result.height,225);
 assert.equal(result.x+result.width,origin.x+origin.width);
 assert.equal(result.y+result.height,origin.y+origin.height);
});
