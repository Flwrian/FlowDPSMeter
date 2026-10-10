import {readFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import ts from 'typescript';
const compile=name=>'data:text/javascript;base64,'+Buffer.from(ts.transpileModule(readFileSync(new URL('../src/'+name,import.meta.url),'utf8'),{compilerOptions:{module:ts.ModuleKind.ES2022,target:ts.ScriptTarget.ES2020}}).outputText.replace("'./view-dom'",JSON.stringify('data:text/javascript,export function patchText(){}'))).toString('base64');
const {buildSeries,groupTimeline,timelineChart}=await import(compile('chart.ts'));
const {sortStats,columns}=await import(compile('stats-table.ts'));
const point=(second,damage,healing=0,damageReceived=0)=>({second,damage,healing,damageReceived});
const row=(name,hps,identityConfirmed=true)=>({member:{name,class:'Clerc'},identityConfirmed,totals:{hps},contribution:0,timeline:[]});
test('sparse intervals keep zero gaps and rates integrate to observed totals',()=>{
 const series=buildSeries([point(0,100),point(3,300)],'damage',4.5);
 assert.deepEqual(series.map(b=>b.amount),[100,0,0,300,0]);
 assert.equal(series.reduce((sum,b)=>sum+b.rate*(b.end-b.start),0),400);
});
test('partial last second and event at exact end keep totals without extra time',()=>{
 const series=buildSeries([point(1,150),point(1.5,50)],'damage',1.5);
 assert.equal(series.length,2);assert.equal(series[1].rate,400);assert.equal(series[1].end,1.5);
 assert.equal(buildSeries([point(1,40)],'damage',.1)[0].rate,40);
});
test('long fights are bounded to 160 buckets without losing amounts',()=>{
 const points=Array.from({length:10001},(_,i)=>point(i,1));
 const series=buildSeries(points,'damage',10000);
 assert.ok(series.length<=160);assert.equal(series.reduce((s,b)=>s+b.amount,0),10001);
});
test('collective timelines retain damage/heal/received separately and do not mutate inputs',()=>{
 const rows=[{...row('Tank',0),timeline:[point(0,50,0,20)]},{...row('Heal',10),timeline:[point(0,0,100),point(1,0,30)]},{...row('Unknown',0,false),timeline:[point(0,999)]}];
 const before=JSON.stringify(rows);
 assert.deepEqual(groupTimeline(rows),[point(0,50,100,20),point(1,0,30)]);assert.equal(JSON.stringify(rows),before);
});
test('all columns exist; healer and zero-damage member stay in sorted rows; unknown last both ways',()=>{
 const rows=[row('Idle',0),row('Heal',100),row('Unknown',0,false)];
 assert.equal(columns.length,10);
 assert.deepEqual(sortStats(rows,'hps',true).map(r=>r.member.name),['Heal','Idle','Unknown']);
 assert.deepEqual(sortStats(rows,'hps',false).map(r=>r.member.name),['Idle','Heal','Unknown']);
 assert.equal(rows[0].member.name,'Idle');
});
test('multiple chart SVGs use no shared gradient IDs and expose interval values',()=>{
 const svg=timelineChart([point(0,100)],'damage',3);
 assert.ok(!svg.includes('id="fill"'));assert.ok(svg.includes('<title>0–1 s'));assert.ok(svg.includes('100 au total'));
 assert.ok(timelineChart([],'healing',3).includes('Aucune série'));
});
