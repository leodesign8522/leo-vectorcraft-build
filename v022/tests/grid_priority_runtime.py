import hashlib,json,math,sys
from pathlib import Path
from native_control import NativeControl

exe=Path(sys.argv[1]).resolve()
with NativeControl(exe) as a:
    a.cmd('prefs.set',values={'gridlineEvery':20,'gridSubdivisions':1})
    def flag(key,command,on):
        if a.call('ui.inspect')['ui']['view'][key]!=on:a.cmd(command)
    def setup(grid,smart):
        a.call('ui.set',{'smartGuides':smart})
        flag('snap_to_pixel','view.snapToPixel',False)
        flag('snap_to_grid','view.snapToGrid',grid)
        a.cmd('view.setZoom',zoom=100,center=[400,350])
    def events(items,mods=None):return a.call('ui.pointer',{'events':[{'kind':k,'x':p[0],'y':p[1]} for k,p in items],'mods':mods or {}})
    records=[]
    for mode in ['node','move','resize','rotate']:
        result=None
        for smart in [False,True]:
            a.cmd('file.new',width=1400,height=1000)
            if mode=='node':
                id=a.cmd('shape.line',x1=300,y1=340,x2=380,y2=340)['id']
                a.cmd('shape.line',x1=390,y1=300,x2=410,y2=360)
                a.cmd('select.anchors',id=id,anchors=[[0,1]],mode='set')
                tool='directSelection';start=[380,340];end=[402,340]
            else:
                id=a.cmd('shape.rectangle',x=400,y=300,width=300,height=200)['id']
                a.cmd('shape.line',x1=203.25,y1=100,x2=203.25,y2=800)
                a.cmd('shape.line',x1=700,y1=300,x2=300,y2=100)
                a.cmd('select.set',ids=[id]);tool='rotate' if mode=='rotate' else 'selection'
                start,end={'move':([550,400],[354,400]),'resize':([400,400],[207.25,400]),'rotate':([400,300],[427,184])}[mode]
            setup(True,smart);a.call('ui.tool.select',{'tool':tool})
            if mode=='rotate':events([('down',[700,300]),('up',[700,300])])
            events([('down',start),('drag',end),('up',end)])
            n=a.node(id)
            if result is None:result=n
            else:assert n==result,f'{mode}: Smart Guides changed grid snapping'
            if mode=='node':assert n['kind']['path']['subpaths'][0]['anchors'][1]['p']==[400,340]
            if mode in ['move','resize']:assert abs(a.bounds(id)['x']-200)<1e-7
            records.append({'tool':mode,'smart_guides':smart,'grid_priority':True})
    # A previously held geometric capture releases as soon as grid is enabled.
    a.cmd('file.new',width=1400,height=1000)
    id=a.cmd('shape.line',x1=300,y1=340,x2=380,y2=340)['id']
    a.cmd('shape.line',x1=390,y1=300,x2=410,y2=360)
    a.cmd('select.anchors',id=id,anchors=[[0,1]],mode='set');setup(False,True)
    a.call('ui.tool.select',{'tool':'directSelection'})
    def endpoint():return a.node(id)['kind']['path']['subpaths'][0]['anchors'][1]['p']
    events([('down',[380,340]),('drag',[402,340])],{'shift':True})
    assert math.dist(endpoint(),[403.333333333333,340])<1e-6
    flag('snap_to_grid','view.snapToGrid',True);events([('drag',[402,340])],{'shift':True});assert endpoint()==[400,340]
    flag('snap_to_grid','view.snapToGrid',False);events([('drag',[402,340]),('up',[402,340])],{'shift':True})
    assert math.dist(endpoint(),[403.333333333333,340])<1e-6
    assert a.call('ui.inspect')['ui']['view']['smart_guides'] is True,'Grid must not overwrite Smart Guides preference'
    a.cmd('edit.undo');assert endpoint()==[380,340]
    output={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':records,
            'grid_suspends_smart_geometry':True,'geometry_restores_after_grid_off':True,
            'mid_drag_capture_released':True,'smart_preference_preserved':True,'undo':True}
    (exe.parent/'GRID_PRIORITY_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8');print(json.dumps(output))
