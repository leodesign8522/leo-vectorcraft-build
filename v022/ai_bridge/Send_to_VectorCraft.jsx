/* Leo Design AI Bridge v0.1.0: send selected full objects; keep the source unchanged. */
(function(){
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

    var testing=typeof LEO_BRIDGE_TEST_MODE!=='undefined' && LEO_BRIDGE_TEST_MODE;
    var source=null,exchange=null,oldCoordinates=app.coordinateSystem,oldInteraction=app.userInteractionLevel;
    try {
        if(!app.documents.length)throw Error('请先打开 Illustrator 文档并选择对象。');
        source=app.activeDocument;
        var chosen=source.selection;
        if(!(chosen instanceof Array) || !chosen.length)throw Error('请用选择工具选中完整对象或编组，再发送。文字编辑中的字符选择暂不支持。');
        var roots=[],i,j;
        for(i=0;i<chosen.length;i++){
            var ancestor=chosen[i].parent,covered=false;
            while(ancestor && ancestor.typename!=='Document'){
                for(j=0;j<chosen.length;j++)if(ancestor===chosen[j])covered=true;
                ancestor=ancestor.parent;
            }
            if(!covered)roots.push(chosen[i]);
        }
        var allowed={PathItem:true,CompoundPathItem:true,GroupItem:true,TextFrame:true,RasterItem:true,PlacedItem:true};
        function validate(item){
            if(!allowed[item.typename])throw Error('第一版尚不支持 '+item.typename+'，请保留原稿并单独处理该对象。');
            if(item.typename==='TextFrame' && item.kind===TextType.PATHTEXT)throw Error('路径文字暂不支持保持活文字往返。');
            if(item.typename==='GroupItem')for(var k=0;k<item.pageItems.length;k++)if(item.pageItems[k].parent===item)validate(item.pageItems[k]);
        }
        for(i=0;i<roots.length;i++){
            if(roots[i].parent.typename!=='Layer')throw Error('请选中最外层完整编组或复合路径后发送，避免丢失父级蒙版和透明度。');
            validate(roots[i]);
        }
        var root=typeof LEO_BRIDGE_TEST_ROOT!=='undefined'?new Folder(LEO_BRIDGE_TEST_ROOT):
            new Folder($.getenv('LOCALAPPDATA')+'/Leo Design/AI Bridge');
        if(!root.exists)throw Error('未配置接收器，请先安装新版 Leo Design Core Bridge。');
        var config=new File(root.fsName+'/receiver.txt');config.encoding='UTF-8';
        if(!config.open('r'))throw Error('找不到 VectorCraft 接收器配置，请重新运行安装包。');
        var executable=config.read().replace(/^\uFEFF/,'').replace(/^\s+|\s+$/g,'');config.close();
        var receiver=new File(executable);
        if(!testing && !receiver.exists)throw Error('配置中的 VectorCraft 不存在，请重新安装 Core Bridge。');
        var inbox=new Folder(root.fsName+'/inbox');if(!inbox.exists && !inbox.create())throw Error('无法创建接收目录。');
        var job=new Folder(inbox.fsName+'/'+new Date().getTime()+'_'+Math.floor(Math.random()*1000000000));
        if(!job.create())throw Error('无法创建本次发送任务。');
        app.coordinateSystem=CoordinateSystem.DOCUMENTCOORDINATESYSTEM;
        app.userInteractionLevel=UserInteractionLevel.DONTDISPLAYALERTS;
        if((Number(source.scaleFactor)||1)!==1)throw Error('第一版暂不支持 Illustrator 大画布比例文档。');
        var board=source.artboards.getActiveArtboardIndex();
        if(board<0 || board>=source.artboards.length)board=0;
        var rect=source.artboards[board].artboardRect,originalRuler=source.rulerOrigin;
        var ruler=[originalRuler[0],originalRuler[1]];
        // Freeze source bounds before switching documents; DOM ruler context matters.
        var records=[];
        for(i=0;i<roots.length;i++){var b=roots[i].geometricBounds;records.push({item:roots[i],position:[b[0],b[1]],z:roots[i].zOrderPosition});}
        records.sort(function(a,b){return b.z-a.z;});
        exchange=app.documents.add(source.documentColorSpace,rect[2]-rect[0],rect[1]-rect[3]);
        exchange.artboards[0].artboardRect=rect;exchange.artboards[0].name=source.artboards[board].name;
        exchange.artboards.setActiveArtboardIndex(0);exchange.rulerOrigin=ruler;
        // Different assigned profiles cannot silently fall back to the working space.
        var sourceProfile='',targetProfile='';
        try{sourceProfile=String(source.colorProfileName||'');targetProfile=String(exchange.colorProfileName||'');}catch(profileUnavailable){}
        if(sourceProfile && targetProfile && sourceProfile!==targetProfile)throw Error('来源与工作空间 ICC 不同，第一版无法保证配置文件继承，已停止发送。');
        var initial=exchange.layers[0];
        function copyLayer(from,parent){
            var relevant=[],k;
            for(k=0;k<records.length;k++)if(records[k].item.parent===from)relevant.push({record:records[k],z:records[k].z,layer:false});
            for(k=0;k<from.layers.length;k++)relevant.push({source:from.layers[k],z:from.layers[k].zOrderPosition,layer:true});
            relevant.sort(function(a,b){return b.z-a.z;});
            var to=parent.layers.add(),count=0;to.name=from.name;
            for(k=0;k<relevant.length;k++){
                var entry=relevant[k];
                if(entry.layer)count+=copyLayer(entry.source,to);
                else{
                    exchange.activate();
                    var copy=entry.record.item.duplicate(to,ElementPlacement.PLACEATBEGINNING),bounds=copy.geometricBounds;
                    copy.translate(entry.record.position[0]-bounds[0],entry.record.position[1]-bounds[1]);count++;
                }
            }
            if(!count){to.remove();return 0;}
            to.opacity=from.opacity;to.blendingMode=from.blendingMode;to.printable=from.printable;to.color=from.color;
            return count;
        }
        var copied=0;
        for(i=source.layers.length-1;i>=0;i--)copied+=copyLayer(source.layers[i],exchange);
        if(copied!==records.length)throw Error('选中对象数量校验失败，已停止发送。');
        initial.remove();
        var output=new File(job.fsName+'/selection.ai'),options=new IllustratorSaveOptions();
        options.pdfCompatible=true;options.compressed=true;options.embedLinkedFiles=true;
        // Exchange numeric process colours; do not silently embed an unknown profile.
        options.embedICCProfile=false;
        exchange.saveAs(output,options);
        if(!output.exists || output.length===0)throw Error('Illustrator 未生成交换文件。');
        exchange.close(SaveOptions.DONOTSAVECHANGES);exchange=null;source.activate();
        var ready=new File(job.fsName+'/ready.tmp');
        write(ready.fsName,{version:1,objects:copied,sourceName:String(source.name),mode:source.documentColorSpace===DocumentColorSpace.CMYK?'cmyk':'rgb',
            colorPolicy:'numeric-working-space',profile:sourceProfile});
        if(!ready.rename('ready.json'))throw Error('无法提交发送任务。');
        var heartbeat=new File(root.fsName+'/heartbeat.txt'),alive=false;
        if(heartbeat.open('r')){var stamp=Number(heartbeat.read());heartbeat.close();alive=isFinite(stamp)&&new Date().getTime()-stamp<15000&&new Date().getTime()>=stamp;}
        if(!testing && !alive && !receiver.execute())throw Error('已保存发送内容，但无法启动 VectorCraft。请手动打开新版接收。');
        return json({pass:true,job:job.fsName,objects:copied,mode:source.documentColorSpace===DocumentColorSpace.CMYK?'cmyk':'rgb'});
    } catch(error) {
        if(testing)throw error;
        alert('发送到 VectorCraft 失败：\n'+String(error));
        return json({pass:false,error:String(error)});
    } finally {
        if(exchange)exchange.close(SaveOptions.DONOTSAVECHANGES);
        app.coordinateSystem=oldCoordinates;app.userInteractionLevel=oldInteraction;
        if(source)source.activate();
    }
})();
