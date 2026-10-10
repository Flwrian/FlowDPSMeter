export type RefreshRate=4|10|20;
export function readRefreshRate(value:string|null):RefreshRate {
 return value==='4'?4:value==='20'?20:10;
}
export function pollInterval(rate:RefreshRate,hidden:boolean,details:boolean):number {
 return hidden?1000:details?Math.max(250,1000/rate):1000/rate;
}
export function pollDelay(interval:number,elapsed:number):number {return Math.max(8,interval-elapsed);}
