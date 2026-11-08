"""Actual A shortcut and pointer regressions using isolated synthetic groups."""
import hashlib,json,math,sys,time
from pathlib import Path
from native_control import NativeControl
exe=Path(sys.argv[1]).resolve()
checks=[]
with NativeControl(exe) as a:
    def select(*ids):a.cmd('select.set',ids=list(ids))
    def pointer(items,mods=None):a.call('ui.pointer',{'events':[{'kind':k,'x':p[0],'y':p[1]} for k,p in items],'mods':mods or {}})
    def fixture(nested,grid=False):
        a.cmd('file.new',width=1400,height=1000)
        a.cmd('prefs.set',values={'gridlineEvery':20,'gridSubdivisions':1})
        a.call('ui.set',{'smartGuides':False})
        for k,c,on in [('snap_to_grid','view.snapToGrid',grid),('snap_to_pixel','view.snapToPixel',False)]:
            if a.call('ui.inspect')['ui']['view'][k]!=on:a.cmd(c)
        circle=a.cmd('shape.ellipse',x=200,y=200,width=400,height=400)['id']
        inner=a.cmd('shape.rectangle',x=320,y=320,width=80,height=80)['id']
        select(circle,inner);group=a.cmd('object.group')['id'];ids=[circle,inner]
        if nested:
            extra=a.cmd('shape.rectangle',x=700,y=200,width=40,height=40)['id'];ids.append(extra)
            select(group,extra);group=a.cmd('object.group')['id']
        a.call('ui.tool.select',{'tool':'selection'});a.call('ui.key',{'key':'A','text':'a'})
        until=time.monotonic()+2
        while a.call('ui.inspect')['tool']!='directSelection' and time.monotonic()<until:time.sleep(.03)
        assert a.call('ui.inspect')['tool']=='directSelection'
        return circle,inner,group,ids
    def shifted(old,new):
        assert abs(old['width']-new['width'])<1e-7 and abs(old['height']-new['height'])<1e-7,(old,new)
        return new['x']-old['x'],new['y']-old['y']
    def anchor_points(id):
        return [tuple(a.get(k,a['p'])) for sp in a.node(id)['kind']['path']['subpaths']
                for a in sp['anchors'] for k in ['p','in','out']]
    for nested in [False,True]:
        for part,start in [('anchor',(400,600)),('fill',(260,380)),('curve',(258.5786437627,258.5786437627))]:
            for mode,mods,grid in [('normal',{},False),('smart',{},False),('grid',{},True),('shift',{'shift':True},False),('alt',{'alt':True},False)]:
                circle,inner,group,ids=fixture(nested,grid)
                if mode=='smart':a.call('ui.set',{'smartGuides':True})
                before={id:a.bounds(id) for id in ids};points={id:anchor_points(id) for id in ids}
                original=a.node(group);b=a.bounds(group)
                end=(start[0]+53,start[1]+47)
                # Shift before a fill press is the native toggle-selection gesture.
                # Add it after pressing to test a constrained drag of the existing group.
                if mode=='shift':
                    pointer([('down',start)])
                    pointer([('drag',end),('up',end)],mods)
                else:pointer([('down',start),('drag',end),('up',end)],mods)
                selected=a.call('document.inspect')['selection']
                if mode=='alt':
                    assert a.node(group)==original,'Alt changed the original group'
                    assert len(selected)==1 and selected[0]!=group
                    delta=shifted(b,a.bounds(selected[0]));assert math.dist(delta,(53,47))<1e-6
                else:
                    assert selected==[group],selected
                    deltas=[shifted(before[id],a.bounds(id)) for id in ids]
                    assert all(math.dist(d,deltas[0])<1e-6 for d in deltas),deltas
                    for id in ids:
                        after_points=anchor_points(id);assert len(after_points)==len(points[id])
                        assert all(math.dist((p[0]+deltas[0][0],p[1]+deltas[0][1]),q)<1e-6
                                   for p,q in zip(points[id],after_points)),'An anchor or Bezier handle deformed'
                    assert math.hypot(*deltas[0])>10
                    if mode=='normal':assert math.dist(deltas[0],(53,47))<1e-6
                    if mode=='shift':assert abs(abs(deltas[0][0])-abs(deltas[0][1]))<1e-6
                after=a.node(selected[0]);a.cmd('edit.undo');assert a.node(group)==original
                a.cmd('edit.redo');assert a.node(selected[0])==after
                checks.append({'nested':nested,'part':part,'mode':mode,'pass':True})
    circle,inner,group,ids=fixture(True);old=a.bounds(circle);inside=a.node(inner)
    pointer([('down',(400,600)),('up',(400,600))])
    assert a.call('document.inspect')['selection']==[circle]
    pointer([('down',(400,600)),('drag',(400,680)),('up',(400,680))])
    assert a.bounds(circle)['height']>old['height']+79 and a.node(inner)==inside
    output={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':checks,
            'actual_A_key':True,'group_rigid_translation':True,'alt_original_unchanged':True,
            'all_anchor_handles_translate':True,'click_then_single_anchor_edit_preserved':True,'undo_redo':True}
    (exe.parent/'WHOLE_SELECTION_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8')
    print(json.dumps(output))
