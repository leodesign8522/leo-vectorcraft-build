import hashlib,json,sys,time
from pathlib import Path
from native_control import NativeControl

exe=Path(sys.argv[1]).resolve();root=exe.parent
with NativeControl(exe) as a:
    a.cmd('file.new',width=1400,height=1000)
    def make(x,y,text,width=420):return a.cmd('text.create',x=x,y=y,text=text,size=80,area={'width':width,'height':600})['id']
    def fit(id):return a.node(id)['kind'].get('area',{}).get('fit','none')
    def glyph(id):return a.node(id)['kind'].get('leoGlyphBounds',False)
    def select(*ids):a.cmd('select.set',ids=list(ids))
    def set_mode(id,mode,on,height=None):
        select(id);p={'fit':mode}
        if height is not None:p['height']=height
        a.cmd('text.areaOptions',**p);a.cmd('type.glyphSelectionBounds',on=on)
    if '--make-legacy' in sys.argv:
        id=make(100,200,'Legacy\nEditable');set_mode(id,'none',False,500)
        a.call('app.save',{'path':str(root/'LEGACY_TEXT_MODES.vectorcraft'),'wait':True})
        (root/'LEGACY_TEXT_MODES_NODE.json').write_text(json.dumps({'id':id,'node':a.node(id)},indent=2),encoding='utf-8')
        print(json.dumps({'legacy_fixture':True,'fixed_height':True,'glyph_bounds_off':True}));sys.exit(0)
    source=make(100,200,'AB',240);target=make(600,220,'Target\nEditable\nText')
    assert fit(source)=='autoHeight' and fit(target)=='autoHeight' and glyph(source) and glyph(target)
    set_mode(source,'none',False,140)
    def sample():
        select(target);a.call('ui.tool.select',{'tool':'selection'});a.call('ui.key',{'key':'I','text':'i'})
        deadline=time.monotonic()+2
        while a.call('ui.inspect')['tool']!='eyedropper' and time.monotonic()<deadline:time.sleep(.05)
        a.call('ui.pointer',{'events':[{'kind':'down','x':135,'y':215},{'kind':'up','x':135,'y':215}]})
        assert a.call('ui.inspect')['tool']=='eyedropper','I key did not activate Eyedropper'
    before=a.node(target);frame=a.bounds(target);source_before=a.node(source)
    sample();assert fit(target)=='none' and not glyph(target);assert a.node(source)==source_before
    assert a.node(target)['kind']['runs'][0]['text']==before['kind']['runs'][0]['text']
    assert a.node(target)['kind']['xf']==before['kind']['xf'];assert a.bounds(target)==frame
    after=a.node(target);a.cmd('edit.undo');assert a.node(target)==before;a.cmd('edit.redo');assert a.node(target)==after
    set_mode(target,'none',False,550);set_mode(source,'autoHeight',True)
    before=a.node(target);frame=a.bounds(target);source_before=a.node(source);source_height=a.bounds(source)['height']
    sample();assert fit(target)=='autoHeight' and glyph(target);assert a.node(source)==source_before
    assert a.bounds(target)['width']==frame['width'] and a.bounds(target)['y']==frame['y']
    assert a.node(target)['kind']['xf']==before['kind']['xf']
    assert source_height<a.bounds(target)['height']<frame['height'],'height must follow target content, not source frame'
    assert a.node(target)['kind']['runs'][0]['text']==before['kind']['runs'][0]['text']
    enabled=a.node(target);a.cmd('edit.undo');assert a.node(target)==before;a.cmd('edit.redo');assert a.node(target)==enabled
    # Independent options can keep object modes while sampling regular styles.
    set_mode(source,'none',False,140);before=a.node(target)
    a.cmd('eyedropper.setOptions',pickUp={'textFrameMode':False,'glyphBounds':False})
    sample();assert fit(target)=='autoHeight' and glyph(target)
    a.cmd('eyedropper.setOptions',pickUp={'textFrameMode':True,'glyphBounds':True})
    if '--capture' in sys.argv:
        a.call('ui.tool.select',{'tool':'selection'});select(source,target)
        a.call('ui.menu.invoke',{'command':'window.panel.dock','params':{'panel':'paragraph','onto':'character','collapsed':False}})
        a.call('ui.menu.invoke',{'command':'window.panel.activate','params':{'panel':'paragraph'}})
        a.call('ui.set',{'status':''});time.sleep(.5)
        a.call('ui.screenshot',{'path':str(root/'TEXT_MODES_MIXED.png'),'data':False})
        a.cmd('text.areaOptions',fit='autoHeight');time.sleep(.3);a.call('ui.screenshot',{'path':str(root/'TEXT_MODES_AUTO.png'),'data':False})
        a.cmd('text.areaOptions',fit='none');time.sleep(.3);a.call('ui.screenshot',{'path':str(root/'TEXT_MODES_FIXED.png'),'data':False})
    # New code must not retroactively turn old objects on.
    legacy=root/'LEGACY_TEXT_MODES.vectorcraft';expected=root/'LEGACY_TEXT_MODES_NODE.json'
    assert legacy.exists() and expected.exists(),'Supply the old-version synthetic fixture in this isolated folder'
    a.call('app.open',{'path':str(legacy)});old=json.loads(expected.read_text(encoding='utf-8'))
    assert a.node(old['id'])==old['node'];assert fit(old['id'])=='none' and not glyph(old['id'])
    output={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'new_defaults_on':True,
            'actual_I_key_and_pointer':True,'sample_off_and_fixed':True,'sample_on_and_auto':True,
            'source_unchanged':True,'target_content_position_width_preserved':True,'target_height_fitted_to_own_content':True,
            'independent_options':True,'undo_redo':True,'legacy_object_unchanged':True}
    (root/'TEXT_MODES_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8');print(json.dumps(output))

