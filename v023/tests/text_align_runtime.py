"""Verify the real Align-panel command against independent Create Outlines bounds."""
import copy,hashlib,json,math,socket,subprocess,sys,time
from pathlib import Path
exe=Path(sys.argv[1]).resolve();assert (exe.parent/'portable.txt').exists()
startup=subprocess.STARTUPINFO();startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
probe=socket.socket();probe.bind(('127.0.0.1',0));port=probe.getsockname()[1];probe.close()
proc=subprocess.Popen([str(exe),'--new-instance','--control',str(port),'--automation-read-root',str(exe.parent),'--automation-write-root',str(exe.parent)],cwd=exe.parent,startupinfo=startup,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
connection=None
try:
    deadline=time.monotonic()+90
    while time.monotonic()<deadline:
        if proc.poll() is not None:raise RuntimeError('Test app exited')
        try:connection=socket.create_connection(('127.0.0.1',port),timeout=2);break
        except OSError:time.sleep(.5)
    assert connection is not None;connection.settimeout(60);f=connection.makefile('rwb');serial=0
    def call(method,params=None):
        global serial
        serial+=1;f.write((json.dumps({'id':serial,'method':method,'params':params or {}})+'\n').encode());f.flush()
        r=json.loads(f.readline());assert r.get('ok'),r
        return r.get('result')
    def command(name,**p):return call('engine.execute',{'command':name,'params':p})
    def node(id):return command('document.node',id=id)
    def centre(b):return [b['x']+b['width']/2,b['y']+b['height']/2]
    def outline_bounds(id):
        # Only a temporary conversion in a synthetic document; restore editable text.
        old=node(id);selected=call('document.inspect')['selection']
        command('select.set',ids=[id]);command('type.createOutlines')
        b=call('document.inspect')['selectionBounds']
        command('edit.undo');assert node(id)==old
        command('select.set',ids=selected)
        return b
    records=[]
    for area in [False,True]:
        for axis,ui_command in [('horizontal','align.horizontalCenter'),('vertical','align.verticalCenter'),('both','align.horizontalVerticalCenter')]:
            command('file.new',width=1200,height=900)
            key=command('shape.rectangle',x=400,y=300,width=500,height=220)['id']
            command('paint.setFill',color='#efb8ca')
            p={'x':180,'y':200,'text':'B5\nAgjp' if area else 'B5','size':180,'color':'#ffffff'}
            if area:p['area']={'width':650,'height':600}
            id=command('text.create',**p)['id'];command('type.glyphSelectionBounds',on=True)
            old=node(id);target=node(key);before=centre(outline_bounds(id));reference=[650,410]
            command('select.set',ids=[id,key]);command('select.key',id=key)
            # This is the UI command registered to the pictured toolbar/Align button.
            call('ui.menu.invoke',{'command':ui_command})
            moved=node(id);ink=outline_bounds(id);c=centre(ink)
            expected=[reference[0] if axis!='vertical' else before[0],reference[1] if axis!='horizontal' else before[1]]
            passed=math.dist(c,expected)<1e-7
            records.append({'area':area,'axis':axis,'pass':passed,'centre_error':math.dist(c,expected)})
            if '--observe' not in sys.argv:assert passed,records[-1]
            assert node(key)==target
            assert moved['kind']['kind']==old['kind']['kind'] and moved['kind']['runs']==old['kind']['runs']
            expected_node=copy.deepcopy(old);expected_node['kind']['xf']=moved['kind']['xf']
            assert expected_node==moved and moved['kind']['xf'][:4]==old['kind']['xf'][:4], 'Alignment must only translate text'
            command('edit.undo');assert node(id)==old
            command('edit.redo');assert node(id)==moved
            if '--capture' in sys.argv and not area and axis=='both':
                command('select.set',ids=[id]);command('view.setZoom',zoom=100,center=[650,410])
                call('ui.set',{'status':''})
                call('ui.screenshot',{'path':str(exe.parent/'TEXT_ALIGN_CENTRE.png'),'data':False})
    output={'pass':all(r['pass'] for r in records),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),
        'checks':records,'key_unchanged':True,'text_layout_unchanged':True,'undo_redo':True,
        'measurement':'temporary native Create Outlines + geometric bounds; editable text restored'}
    name='TEXT_ALIGN_OBSERVE.json' if '--observe' in sys.argv else 'TEXT_ALIGN_RUNTIME.json'
    (exe.parent/name).write_text(json.dumps(output,indent=2),encoding='utf-8');print(json.dumps(output))
finally:
    if connection is not None:
        try:
            command('file.closeAll')
            for _ in range(8):call('ui.dialog.set',{'field':'discard','value':True});call('ui.dialog.confirm')
        except Exception:pass
        try:call('app.quit');proc.wait(timeout=10)
        except Exception:pass
        connection.close()
    if proc.poll() is None:proc.terminate();proc.wait(timeout=10)
