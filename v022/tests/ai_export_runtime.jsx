/* Real Illustrator assertions; opens/closes only this test's exported fixtures. */
(function(){
 var root=LEO_RUNTIME_ROOT.replace(/\\/g,'/')+'/';
 var old=app.documents.length?app.activeDocument:null,level=app.userInteractionLevel,d=null,results=[];
 function ok(value,msg){if(!value)throw Error(msg);}
 function near(a,b){return Math.abs(a-b)<0.001;}
 try{
  app.userInteractionLevel=UserInteractionLevel.DONTDISPLAYALERTS;
  for(var m=0;m<2;m++){
   var mode=m?'cmyk':'rgb';d=app.open(new File(root+mode+'_EDITABLE.ai'));
   ok(d.documentColorSpace===(m?DocumentColorSpace.CMYK:DocumentColorSpace.RGB),'mode mismatch');
   ok(d.artboards.length===2&&d.artboards[1].name==='Second','artboards changed');
   var ab=d.artboards[1].artboardRect;ok(near(ab[2]-ab[0],200)&&near(ab[1]-ab[3],150),'second artboard size changed');
   ok(d.layers.length===2 && d.layers[0].name==='Editable Text' && d.layers[1].name==='Shapes','layer mismatch');
   ok(d.groupItems.length===1,'group structure mismatch');ok(d.textFrames.length===2,'text became outlines or split lines');
   var area=null,point=null;
   for(var t=0;t<d.textFrames.length;t++){if(d.textFrames[t].kind===TextType.AREATEXT)area=d.textFrames[t];else point=d.textFrames[t];}
   ok(area&&point,'missing area or point text');ok(area.paragraphs.length===2,'paragraphs split');
   ok(point.contents==='Editable ABC 123','point text changed');
   ok(area.contents==='中文段落文字自动换行测试，保留文字框。\r第二段中文文字。','area text changed');
   ok(near(area.textPath.width,210)&&near(area.textPath.height,150),'frame dimensions changed');
   var first=d.artboards[0].artboardRect,anchor=point.anchor,frame=area.textPath.geometricBounds;
   ok(near(anchor[0]-first[0],60)&&near(first[1]-anchor[1],230),'point text position changed');
   ok(near(frame[0]-first[0],60)&&near(first[1]-frame[1],300),'area frame position changed');
   ok(point.textRange.characterAttributes.textFont.name==='ArialMT','point font changed');
   ok(area.textRange.characterAttributes.textFont.name==='MicrosoftYaHei','area font changed');
   var c=area.textRange.characterAttributes.fillColor;
   function checkColor(c){
    if(m)ok(c.typename==='CMYKColor'&&near(c.cyan,10)&&near(c.magenta,20)&&near(c.yellow,30)&&near(c.black,40),'CMYK values changed');
    else ok(c.typename==='RGBColor'&&near(c.red,51)&&near(c.green,102)&&near(c.blue,153),'RGB values changed');
   }
   checkColor(c);checkColor(point.textRange.characterAttributes.fillColor);
   var colored=0;for(var p=0;p<d.pathItems.length;p++)if(d.pathItems[p].filled){checkColor(d.pathItems[p].fillColor);colored++;}
   ok(colored===2,'geometry missing');
   for(var s=0;s<d.spots.length;s++)ok(d.spots[s].name.indexOf('LEO_AI_TEXT_')!==0,'marker spot remained');
   var lines=area.lines.length;area.textPath.width=150;ok(area.lines.length>lines,'area text does not reflow');
   area.textPath.width=210;area.contents=area.contents+'\r新增可编辑文字';
   var o=new IllustratorSaveOptions();o.pdfCompatible=true;o.compressed=true;o.embedICCProfile=true;
   var edited=new File(root+mode+'_EDITED.ai');d.saveAs(edited,o);d.close(SaveOptions.DONOTSAVECHANGES);d=null;
   d=app.open(edited);ok(d.textFrames.length===2,'reopen text count');area=null;for(t=0;t<d.textFrames.length;t++)if(d.textFrames[t].kind===TextType.AREATEXT)area=d.textFrames[t];
   ok(area&&area.contents.indexOf('新增可编辑文字')>=0,'reopen lost edit');ok(d.layers[0].name==='Editable Text'&&d.layers[1].name==='Shapes','reopen layer mismatch');
   results.push(mode+': PASS / editable point + area text / automatic reflow / exact frame / layers + group / exact color values / edit-save-reopen');
   d.close(SaveOptions.DONOTSAVECHANGES);d=null;
  }
  return results.join('\n');
 }finally{if(d)d.close(SaveOptions.DONOTSAVECHANGES);app.userInteractionLevel=level;if(old)old.activate();}
})();
