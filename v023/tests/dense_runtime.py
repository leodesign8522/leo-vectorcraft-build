import json, socket, subprocess, sys, time, hashlib
from pathlib import Path

exe=Path(sys.argv[1]).resolve()
assert (exe.parent/'portable.txt').exists(), 'Runtime checks require isolated portable settings'
startup=subprocess.STARTUPINFO()
startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
startup.wShowWindow = 0
probe=socket.socket(); probe.bind(('127.0.0.1',0)); port=probe.getsockname()[1]; probe.close()
proc=subprocess.Popen([str(exe),'--new-instance','--control',str(port),'--automation-read-root',str(exe.parent),'--automation-write-root',str(exe.parent)],cwd=exe.parent,startupinfo=startup,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
results=[]
connection=None
try:
 deadline=time.monotonic()+90
 while time.monotonic()<deadline:
  if proc.poll() is not None: raise RuntimeError('Application exited before control server became ready')
  try:
   connection=socket.create_connection(('127.0.0.1',port),timeout=2); break
  except OSError: time.sleep(0.5)
 if connection is None: raise TimeoutError('Control server did not become ready')
 connection.settimeout(60)
 f=connection.makefile('rwb'); serial=0
 def call(method,params=None):
  global serial
  serial+=1; f.write((json.dumps({'id':serial,'method':method,'params':params or {}})+'\n').encode()); f.flush()
  reply=json.loads(f.readline())
  if not reply.get('ok'): raise RuntimeError(reply)
  return reply.get('result')
 def command(name,**params): return call('engine.execute',{'command':name,'params':params})
 def node(id): return command('document.node',id=id)
 command('file.new',width=500,height=500)
 command('path.create',d='M10000 10000H10010V10010H10000Z '*5001)
 call('ui.set',{'smartGuides':True})
 if call('ui.inspect')['ui']['view']['snap_to_grid']: command('view.snapToGrid')
 source=command('shape.line',x1=300,y1=340,x2=380,y2=340)['id']
 target=command('shape.line',x1=390,y1=300,x2=410,y2=360)['id']
 call('ui.tool.select',{'tool':'directSelection'})
 command('select.anchors',id=source,anchors=[[0,1]],mode='set')
 mods={'shift':len(sys.argv)<3 or sys.argv[2]!='free'}
 call('ui.pointer',{'events':[{'kind':'down','x':380,'y':340},{'kind':'drag','x':402,'y':340}],'mods':mods})
 preview=node(source)['kind']['path']['subpaths'][0]['anchors']
 x,y=preview[1]['p']
 assert abs(3*x-y-870.0)<1e-6, ('during drag',preview)
 if mods['shift']: assert abs(y-340.0)<1e-7, preview
 call('ui.pointer',{'events':[{'kind':'up','x':402,'y':340}],'mods':mods})
 result=node(source)
 anchors=result['kind']['path']['subpaths'][0]['anchors']
 assert anchors[0]['p']==[300.0,340.0], anchors
 x,y=anchors[1]['p']
 assert abs(3*x-y-870.0)<1e-6, anchors
 if len(sys.argv)<3 or sys.argv[2]!='free': assert abs(y-340.0)<1e-7, anchors
 selection=call('document.inspect')['selection']
 assert selection, 'Shift drag erased the live selection'
 command('file.close')
 call('ui.dialog.set',{'field':'discard','value':True})
 call('ui.dialog.confirm')
 command('file.new',width=1000,height=1000)
 command('path.create',d='M10000 10000H10010V10010H10000Z '*5001)
 a=command('shape.rectangle',x=100,y=100,width=50,height=100)['id']
 b=command('shape.rectangle',x=160,y=100,width=40,height=100)['id']
 command('select.set',ids=[a,b])
 group=command('object.group')['id']
 command('shape.line',x1=300,y1=100,x2=300,y2=200)
 command('select.set',ids=[group])
 before=[node(a),node(b)]
 call('ui.tool.select',{'tool':'selection'})
 call('ui.pointer',{'events':[{'kind':'down','x':125,'y':125},{'kind':'drag','x':224,'y':125}],'mods':mods})
 def group_bounds(): return command('document.node',id=group,summary=True)['bounds']
 expected={'x':200.0,'y':100.0,'width':100.0,'height':100.0}
 assert group_bounds()==expected, ('group during drag',group_bounds())
 for child,want in [(a,{'x':200.0,'y':100.0,'width':50.0,'height':100.0}),
                    (b,{'x':260.0,'y':100.0,'width':40.0,'height':100.0})]:
  assert command('document.node',id=child,summary=True)['bounds']==want, 'Dense group child geometry'
 call('ui.pointer',{'events':[{'kind':'up','x':224,'y':125}],'mods':mods})
 assert group_bounds()==expected,group_bounds()
 moved=[node(a),node(b)]
 command('edit.undo')
 assert [node(a),node(b)]==before, 'Dense group undo'
 command('edit.redo')
 assert [node(a),node(b)]==moved, 'Dense group redo'
 output={'pass':True,'check':'Node and rigid group beyond 20k background anchors snap during drag and on release','shift':mods['shift'],'source_after':result,'selection':selection,'group_bounds':group_bounds(),'group_undo_redo':True}
 (exe.parent/('DENSE_RUNTIME_'+('SHIFT' if mods['shift'] else 'FREE')+'.json')).write_text(json.dumps(output,ensure_ascii=False,indent=2),encoding='utf-8')
 print(json.dumps({'pass':True,'shift':mods['shift'],'anchor':anchors[1]['p'],'stationary_anchor':anchors[0]['p'],'group_bounds':group_bounds(),'group_undo_redo':True}))

finally:
 if connection is not None:
  try:
   call('engine.execute',{'command':'file.close','params':{}})
   call('ui.dialog.set',{'field':'discard','value':True})
   call('ui.dialog.confirm')
   call('app.quit')
   proc.wait(timeout=10)
  except Exception: pass
  connection.close()
 if proc.poll() is None:
  proc.terminate(); proc.wait(timeout=10)
