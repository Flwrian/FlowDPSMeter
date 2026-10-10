// Keep controls, focus, scroll and disclosure state across live refreshes.
const markup = new WeakMap<Element,string>();
export function patchHtml(element:HTMLElement, html:string):void {
 if(markup.get(element)===html)return;
 const open=element.querySelector<HTMLDetailsElement>('details')?.open;
 const scroll=element.querySelector<HTMLElement>('.sheet')?.scrollTop;
 element.innerHTML=html;markup.set(element,html);
 const details=element.querySelector<HTMLDetailsElement>('details');
 if(details&&open!==undefined)details.open=open;
 const sheet=element.querySelector<HTMLElement>('.sheet');
 if(sheet&&scroll!==undefined)sheet.scrollTop=scroll;
}
export function patchText(element:Element|null,text:string):void {
 if(element&&element.textContent!==text)element.textContent=text;
}
