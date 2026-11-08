/* Fixed Illustrator-side program. Document strings are JSON data, never code. */
(function () {
    function parseJSON(s) {
        var i=0,depth=0;
        function ws(){while(i<s.length && /\s/.test(s.charAt(i)))i++;}
        function value(){
            ws();if(++depth>100)throw Error('JSON nesting limit');var c=s.charAt(i),v,k,a;
            if(c==='"'){
                i++;v='';while(i<s.length){c=s.charAt(i++);if(c==='"'){depth--;return v;}
                    if(c==='\\'){c=s.charAt(i++);if(c==='u'){a=s.substr(i,4);if(!/^[0-9a-fA-F]{4}$/.test(a))throw Error('Invalid Unicode escape');v+=String.fromCharCode(parseInt(a,16));i+=4;}
                        else {a={'"':'"','\\':'\\','/':'/','b':'\b','f':'\f','n':'\n','r':'\r','t':'\t'};if(!a.hasOwnProperty(c))throw Error('Invalid escape');v+=a[c];}}
                    else {if(c.charCodeAt(0)<32)throw Error('Invalid JSON string');v+=c;}}
                throw Error('Unterminated JSON string');
            }
            if(c==='['){i++;v=[];ws();if(s.charAt(i)===']'){i++;depth--;return v;}while(true){v.push(value());ws();c=s.charAt(i++);if(c===']')break;if(c!==',')throw Error('Invalid array');}depth--;return v;}
            if(c==='{'){i++;v={};ws();if(s.charAt(i)==='}'){i++;depth--;return v;}while(true){ws();if(s.charAt(i)!=='"')throw Error('Invalid object key');k=value();if(k==='__proto__'||k==='constructor'||k==='prototype')throw Error('Unsafe JSON key');ws();if(s.charAt(i++)!==':')throw Error('Invalid object');v[k]=value();ws();c=s.charAt(i++);if(c==='}')break;if(c!==',')throw Error('Invalid object');}depth--;return v;}
            a=/^(true|false|null|-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?)/.exec(s.substr(i));
            if(!a)throw Error('Invalid JSON value');i+=a[0].length;depth--;if(a[0]==='true')return true;if(a[0]==='false')return false;if(a[0]==='null')return null;v=Number(a[0]);if(!isFinite(v))throw Error('Non-finite number');return v;
        }
        if(s.charCodeAt(0)===0xFEFF)i++;var v=value();ws();if(i!==s.length)throw Error('Trailing JSON data');return v;
    }
    function json(v){
        if(v===null)return 'null';
        if(typeof v==='string')return '"'+v.replace(/[\\"\u0000-\u001f]/g,function(c){if(c==='\\')return '\\\\';if(c==='"')return '\\"';var h=c.charCodeAt(0).toString(16);return '\\u'+('0000'+h).slice(-4);})+'"';
        if(typeof v!=='object')return String(v);
        var a=[],k;if(v instanceof Array){for(k=0;k<v.length;k++)a.push(json(v[k]));return '['+a.join(',')+']';}
        for(k in v)if(v.hasOwnProperty(k))a.push(json(k)+':'+json(v[k]));return '{'+a.join(',')+'}';
    }
    function read(path){var f=new File(path);f.encoding='UTF-8';if(!f.open('r'))throw Error('Cannot read export manifest');var s=f.read();f.close();if(s.length>16000000)throw Error('Export manifest too large');return parseJSON(s);}
    function write(path,v){var f=new File(path);f.encoding='UTF-8';if(!f.open('w'))throw Error('Cannot write export report');f.write(json(v));f.close();}
    function norm(s){return String(s||'').toLowerCase().replace(/[\s_\-]/g,'');}
    var fonts={};
    function font(style){
        var family=norm(style.font_family),wanted=norm(style.font_style||'Regular'),key='$'+family+'|'+wanted;
        if(fonts.hasOwnProperty(key))return fonts[key];
        var regular=/^(regular|normal|roman|常规|標準|标准)$/.test(wanted),fallback=null;
        for(var i=0;i<app.textFonts.length;i++){
            var f=app.textFonts[i];
            if(norm(f.family)===family && norm(f.style)===wanted)return fonts[key]=f;
            if(regular && norm(f.name)===family)fallback=f;
        }
        if(fallback)return fonts[key]=fallback;
        throw Error('Illustrator 缺少字体：'+style.font_family+' / '+style.font_style);
    }
    function color(p,doc){
        if(!p || p.type==='none')return new NoColor();
        if(p.type!=='solid')throw Error('当前活文字导出需要普通填色或专色');
        if(p.swatch){try{var spot=doc.spots.getByName(p.swatch);var sc=new SpotColor();sc.spot=spot;sc.tint=(p.tint===undefined?1:p.tint)*100;return sc;}catch(ignore){}}
        var c=p.color,v;
        if(c.model==='rgb'){v=new RGBColor();v.red=c.r*255;v.green=c.g*255;v.blue=c.b*255;}
        else if(c.model==='cmyk'){v=new CMYKColor();v.cyan=c.c*100;v.magenta=c.m*100;v.yellow=c.y*100;v.black=c.k*100;}
        else if(c.model==='gray'){v=new GrayColor();v.gray=c.k*100;}
        else if(c.model==='lab'){v=new LabColor();v.l=c.l;v.a=c.a;v.b=c.b;}
        else throw Error('Unsupported text colour model');return v;
    }
    function character(a,s,doc,scale,range){
        a.textFont=font(s);a.size=s.size/scale;a.tracking=s.tracking||0;
        a.horizontalScale=s.h_scale===undefined?100:s.h_scale;a.verticalScale=s.v_scale===undefined?100:s.v_scale;
        a.baselineShift=(s.baseline_shift||0)/scale;a.rotation=-(s.rotation||0);
        a.autoLeading=s.leading===undefined||s.leading===null;if(!a.autoLeading)a.leading=s.leading/scale;
        a.fillColor=color(s.fill,doc);a.strokeColor=color(s.stroke,doc);a.strokeWeight=(s.stroke_width||0)/scale;
        a.underline=!!s.underline;a.strikeThrough=!!s.strikethrough;
        a.noBreak=!!s.no_break;a.overprintFill=!!s.overprint_fill;a.overprintStroke=!!s.overprint_stroke;
        if(s.opticalKerning)a.kerningMethod=AutoKernType.OPTICAL;
        if(s.kerning!==undefined && s.kerning!==null && range)range.kerning=s.kerning;
        if(s.all_caps)a.capitalization=FontCapsOption.ALLCAPS;
    }
    function paragraph(a,p,scale){
        var j={Auto:Justification.LEFT,Left:Justification.LEFT,Center:Justification.CENTER,Right:Justification.RIGHT,JustifyLeft:Justification.FULLJUSTIFYLASTLINELEFT,JustifyCenter:Justification.FULLJUSTIFYLASTLINECENTER,JustifyRight:Justification.FULLJUSTIFYLASTLINERIGHT,JustifyAll:Justification.FULLJUSTIFY};
        a.justification=j[p.justify||'Left'];a.leftIndent=(p.left_indent||0)/scale;a.rightIndent=(p.right_indent||0)/scale;
        a.firstLineIndent=(p.first_line_indent||0)/scale;a.spaceBefore=(p.space_before||0)/scale;a.spaceAfter=(p.space_after||0)/scale;
        a.hyphenation=!!p.hyphenate;a.everyLineComposer=p.composer!=='singleLine';
        if(p.justification && p.justification.auto_leading!==undefined)a.autoLeadingAmount=p.justification.auto_leading;
    }
    function path(parent,data,scale){
        if(data.subpaths.length!==1 || !data.subpaths[0].closed)throw Error('区域文字框需要单一闭合路径');
        var sp=data.subpaths[0],p=parent.pathItems.add(),points=[],i;
        for(i=0;i<sp.anchors.length;i++)points.push([sp.anchors[i].p[0]/scale,-sp.anchors[i].p[1]/scale]);
        p.setEntirePath(points);p.closed=true;p.filled=false;p.stroked=false;
        for(i=0;i<sp.anchors.length;i++){
            var a=sp.anchors[i],pin=a['in']||a.p,pout=a.out||a.p;
            p.pathPoints[i].leftDirection=[pin[0]/scale,-pin[1]/scale];p.pathPoints[i].rightDirection=[pout[0]/scale,-pout[1]/scale];
        }
        return p;
    }
    function createText(doc,marker,record,scale){
        var node=record.node,t=node.kind,kind=t.kind.kind,parent=marker.parent,tf,content='',i;
        if(kind==='point')tf=doc.textFrames.pointText([0,0],t.vertical?TextOrientation.VERTICAL:TextOrientation.HORIZONTAL);
        else if(kind==='area'){
            tf=doc.textFrames.areaText(path(parent,t.kind.frame,scale),t.vertical?TextOrientation.VERTICAL:TextOrientation.HORIZONTAL);
            var area=t.area||{};tf.columnCount=area.columns||1;tf.rowCount=area.rows||1;
            tf.columnGutter=(area.gutter===undefined?18:area.gutter)/scale;tf.rowGutter=(area.rowGutter===undefined?(area.gutter===undefined?18:area.gutter):area.rowGutter)/scale;
            tf.flowLinksHorizontally=area.flow==='rows';tf.spacing=(area.inset||0)/scale;
        } else throw Error('当前活文字导出暂不支持路径文字');
        tf.move(marker,ElementPlacement.PLACEBEFORE);
        for(i=0;i<t.runs.length;i++)content+=t.runs[i].text.replace(/\n/g,'\r');tf.contents=content;
        if(t.runs.length)character(tf.textRange.characterAttributes,t.runs[0].style,doc,scale,tf.textRange);
        if(t.runs.length>1){
            var ends=[],end=0;for(i=0;i<t.runs.length;i++){end+=t.runs[i].text.length;ends.push(end);}
            var at=0,run=0;
            for(i=0;i<tf.characters.length;i++){
                while(run+1<t.runs.length && at>=ends[run])run++;
                if(run>0)character(tf.characters[i].characterAttributes,t.runs[run].style,doc,scale,tf.characters[i]);
                at+=tf.characters[i].contents.length;
            }
        }
        for(i=0;i<tf.paragraphs.length;i++)paragraph(tf.paragraphs[i].paragraphAttributes,(t.paras&&t.paras[i])||t.para||{},scale);
        var x=t.xf,m=app.getIdentityMatrix(),anchor=marker.geometricBounds;
        m.mValueA=x[0];m.mValueB=-x[1];m.mValueC=-x[2];m.mValueD=x[3];m.mValueTX=anchor[0];m.mValueTY=anchor[1];
        tf.transform(m,true,true,true,true,100,Transformation.DOCUMENTORIGIN);
        tf.name=node.name||('Text '+node.id);tf.opacity=(node.opacity===undefined?1:node.opacity)*100;
        tf.hidden=node.visible===false;tf.locked=!!node.locked;marker.remove();return tf;
    }
    function freshDocument(src){
        var scale=Number(src.scaleFactor)||1,b=src.artboards[0].artboardRect;
        var width=(b[2]-b[0])*scale,height=(b[1]-b[3])*scale;
        if(scale>1)width=Math.max(width,16384);
        var dst=app.documents.add(src.documentColorSpace,width,height),initial=dst.layers[0];
        try{
            if((Number(dst.scaleFactor)||1)!==scale)throw Error('Illustrator 大画布比例不一致，已停止导出');
            dst.artboards[0].artboardRect=b;dst.artboards[0].name=src.artboards[0].name;
            for(var i=1;i<src.artboards.length;i++){var ab=dst.artboards.add(src.artboards[i].artboardRect);ab.name=src.artboards[i].name;}
            // Preserve spot/process definitions before cross-document duplication.
            for(i=0;i<src.spots.length;i++){
                var spot=src.spots[i];if(spot.name==='[Registration]' || spot.colorType===ColorModel.REGISTRATION)continue;
                var target;try{target=dst.spots.getByName(spot.name);}catch(missing){target=dst.spots.add();target.name=spot.name;}
                target.colorType=spot.colorType;target.color=spot.color;
            }
            function layer(from,parent){
                var to=parent.layers.add(),locked=from.locked,visible=from.visible,entries=[],j;
                to.name=from.name;from.locked=false;from.visible=true;
                for(j=0;j<from.layers.length;j++)entries.push({item:from.layers[j],layer:true,z:from.layers[j].zOrderPosition});
                for(j=0;j<from.pageItems.length;j++)if(from.pageItems[j].parent===from)entries.push({item:from.pageItems[j],layer:false,z:from.pageItems[j].zOrderPosition});
                entries.sort(function(a,b){return b.z-a.z;});
                for(j=0;j<entries.length;j++){
                    var e=entries[j];if(e.layer)layer(e.item,to);
                    else {var il=e.item.locked,ih=e.item.hidden;e.item.locked=false;e.item.hidden=false;var copy=e.item.duplicate(to,ElementPlacement.PLACEATBEGINNING);copy.hidden=ih;copy.locked=il;}
                }
                to.opacity=from.opacity;to.blendingMode=from.blendingMode;to.printable=from.printable;to.color=from.color;
                to.visible=visible;to.locked=locked;return to;
            }
            for(i=src.layers.length-1;i>=0;i--)layer(src.layers[i],dst);
            initial.remove();dst.activate();return dst;
        }catch(e){dst.close(SaveOptions.DONOTSAVECHANGES);throw e;}
    }
    var manifest=read(LEO_BUNDLE_PATH),previous=app.documents.length?app.activeDocument:null,previousCount=app.documents.length;
    var interaction=app.userInteractionLevel,doc=null,sourceDoc=null,unlocked=[],report={pass:false,version:app.version,texts:[],beforeDocuments:previousCount};
    function unlock(parent){
        if(!parent || parent.typename==='Document')return;
        for(var i=0;i<unlocked.length;i++)if(unlocked[i].item===parent)return;
        unlock(parent.parent);
        var layer=parent.typename==='Layer',state={item:parent,layer:layer,locked:parent.locked,hidden:layer?!parent.visible:parent.hidden};
        unlocked.push(state);parent.locked=false;if(layer)parent.visible=true;else parent.hidden=false;
    }
    try{
        app.userInteractionLevel=UserInteractionLevel.DONTDISPLAYALERTS;
        for(var n=0;n<manifest.texts.length;n++)for(var r=0;r<manifest.texts[n].node.kind.runs.length;r++)font(manifest.texts[n].node.kind.runs[r].style);
        sourceDoc=app.open(new File(manifest.inputAi));
        doc=freshDocument(sourceDoc);sourceDoc.close(SaveOptions.DONOTSAVECHANGES);sourceDoc=null;
        var cmyk=doc.documentColorSpace===DocumentColorSpace.CMYK;
        if(cmyk!==(manifest.mode==='cmyk'))throw Error('导入后的文档颜色模式不一致，已停止导出');
        var scale=Number(doc.scaleFactor)||1,markers={},expected={};
        for(n=0;n<manifest.texts.length;n++)expected['$'+manifest.texts[n].marker]=true;
        for(var p=0;p<doc.pathItems.length;p++){
            var item=doc.pathItems[p];if(item.filled && item.fillColor.typename==='SpotColor'){
                var key='$'+item.fillColor.spot.name;if(expected.hasOwnProperty(key)){if(markers.hasOwnProperty(key))throw Error('Duplicate text marker');markers[key]=item;}
            }
        }
        for(n=0;n<manifest.texts.length;n++){
            var rec=manifest.texts[n],marker=markers['$'+rec.marker];if(!marker)throw Error('文字位置标记缺失：'+rec.marker);
            unlock(marker.parent);marker.locked=false;marker.hidden=false;var tf=createText(doc,marker,rec,scale);
            report.texts.push({id:rec.node.id,name:tf.name,kind:String(tf.kind),contents:tf.contents,font:tf.textRange.characterAttributes.textFont.name,paragraphs:tf.paragraphs.length,lines:tf.lines.length,layer:tf.layer.name});
        }
        for(n=0;n<manifest.texts.length;n++){try{doc.spots.getByName(manifest.texts[n].marker).remove();}catch(ignore){}}
        for(n=unlocked.length-1;n>=0;n--){var state=unlocked[n];if(state.layer)state.item.visible=!state.hidden;else state.item.hidden=state.hidden;state.item.locked=state.locked;}
        report.stage='save';
        var options=new IllustratorSaveOptions();options.pdfCompatible=true;options.compressed=true;options.embedICCProfile=true;options.embedLinkedFiles=true;
        doc.saveAs(new File(manifest.outputAi),options);
        report.stage='saved';
        report.mode=cmyk?'cmyk':'rgb';report.artboards=doc.artboards.length;report.layers=[];
        for(n=0;n<doc.layers.length;n++)report.layers.push(doc.layers[n].name);
        doc.close(SaveOptions.DONOTSAVECHANGES);doc=null;report.pass=true;
    }catch(e){report.error=String(e);throw e;}
    finally{
        if(doc)doc.close(SaveOptions.DONOTSAVECHANGES);
        if(sourceDoc)sourceDoc.close(SaveOptions.DONOTSAVECHANGES);
        app.userInteractionLevel=interaction;if(previous)previous.activate();report.afterDocuments=app.documents.length;
        if(manifest.reportPath)write(manifest.reportPath,report);
    }
    return json(report);
})();
