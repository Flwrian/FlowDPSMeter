import type { Row,Metric,Point } from './types';

const format=new Intl.NumberFormat('fr-FR',{notation:'compact',maximumFractionDigits:1});
const seconds=new Intl.NumberFormat('fr-FR',{maximumFractionDigits:1});
export interface Bucket {start:number;end:number;amount:number;rate:number}

export function groupTimeline(rows:Row[]):Point[] {
 const points=new Map<number,Point>();
 for(const row of rows){
  if(row.identityConfirmed===false)continue;
  for(const point of row.timeline){
   const total=points.get(point.second)??{second:point.second,damage:0,healing:0,damageReceived:0};
   total.damage+=point.damage;total.healing+=point.healing;total.damageReceived+=point.damageReceived;
   points.set(point.second,total);
  }
 }
 return [...points.values()].sort((a,b)=>a.second-b.second);
}

// Keep integer-second capture buckets intact; the last interval can be shorter.
// Integrating the displayed rates recovers the exact observed total.
export function buildSeries(points:Point[],metric:Metric,duration:number,limit=160):Bucket[] {
 const span=Math.max(1,duration),width=Math.max(1,Math.ceil(Math.ceil(span)/Math.max(1,limit)));
 const count=Math.ceil(span/width);
 const buckets=Array.from({length:count},(_,i)=>({start:i*width,end:Math.min(span,(i+1)*width),amount:0,rate:0}));
 for(const point of points){
  if(point.second<0||point.second>span)continue;
  buckets[Math.min(count-1,Math.floor(point.second/width))].amount+=point[metric];
 }
 for(const bucket of buckets)bucket.rate=bucket.amount/(bucket.end-bucket.start);
 return buckets;
}

export function timelineChart(timeline:Point[],metric:Metric,duration:number):string {
 if(!timeline.length)return '<div class="chart-empty">Aucune série temporelle enregistrée.</div>';
 const width=510,height=120,pad=10,span=Math.max(1,duration);
 const buckets=buildSeries(timeline,metric,span),peak=Math.max(...buckets.map(p=>p.rate),1);
 const x=(second:number)=>(pad+second/span*(width-pad*2)).toFixed(2);
 const y=(rate:number)=>(height-pad-rate/peak*(height-pad*2)).toFixed(2);
 const line=buckets.map(p=>`${x(p.start)},${y(p.rate)} ${x(p.end)},${y(p.rate)}`).join(' ');
 const descriptions:Record<Metric,string>={damage:'Dégâts',healing:'Soins rapportés',damageReceived:'Dégâts reçus'};
 return `<div class="chart"><div class="chart-caption"><span>Maximum ${format.format(Math.max(...buckets.map(p=>p.rate)))} / s</span><span>0 → ${seconds.format(span)} s</span></div>
 <svg viewBox="0 0 ${width} ${height}" role="img" aria-label="${descriptions[metric]} par seconde sur une durée commune de ${seconds.format(span)} secondes">
 <polygon points="${pad},${height-pad} ${line} ${width-pad},${height-pad}" fill="var(--accent)" fill-opacity=".12"/>
 <polyline points="${line}" fill="none" stroke="var(--accent)" stroke-width="2" stroke-linejoin="round"/>
 <line x1="10" y1="110" x2="500" y2="110" stroke="#33404e"/>
 ${buckets.map(p=>`<rect x="${x(p.start)}" y="0" width="${((p.end-p.start)/span*(width-pad*2)).toFixed(2)}" height="${height}" fill="transparent"><title>${seconds.format(p.start)}–${seconds.format(p.end)} s : ${format.format(p.rate)} / s · ${format.format(p.amount)} au total</title></rect>`).join('')}</svg></div>`;
}

export function chart(row:Row,metric:Metric,duration:number):string {
 return timelineChart(row.timeline,metric,duration);
}
