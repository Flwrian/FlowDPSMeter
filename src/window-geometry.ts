export interface Geometry {x:number;y:number;width:number;height:number;scale:number}

export function transformGeometry(original:Geometry,mode:string,deltaX:number,deltaY:number,compact=false):Geometry {
 const dx=Math.round(deltaX*original.scale),dy=Math.round(deltaY*original.scale);
 const result={...original};
 const minWidth=Math.ceil((compact?320:420)*original.scale),minHeight=Math.ceil((compact?180:420)*original.scale);
 if(mode==='Move'){result.x+=dx;result.y+=dy;return result;}
 if(mode.includes('East'))result.width=Math.max(minWidth,original.width+dx);
 if(mode.includes('South'))result.height=Math.max(minHeight,original.height+dy);
 if(mode.includes('West')){result.width=Math.max(minWidth,original.width-dx);result.x=original.x+original.width-result.width;}
 if(mode.includes('North')){result.height=Math.max(minHeight,original.height-dy);result.y=original.y+original.height-result.height;}
 return result;
}
