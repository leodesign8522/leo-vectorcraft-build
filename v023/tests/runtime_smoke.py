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
 call('ui.set',{'smartGuides':True})
 settings=call('ui.inspect')['ui']['view']
 desired_grid = not (len(sys.argv)>2 and sys.argv[2]=='grid-off')
 if settings['snap_to_grid'] != desired_grid: command('view.snapToGrid')
 settings=call('ui.inspect')['ui']['view']
 a=command('shape.rectangle',x=100,y=100,width=50,height=100)['id']
 b=command('shape.rectangle',x=160,y=100,width=40,height=100)['id']
 command('select.set',ids=[a,b])
 group=command('object.group')['id']
 before=[node(a),node(b)]
 call('ui.tool.select',{'tool':'selection'})
 call('ui.pointer',{'events':[{'kind':'down','x':125,'y':125},{'kind':'drag','x':424,'y':125},{'kind':'up','x':424,'y':125}]})
 moved=[node(a),node(b)]
 group_bounds=command('document.node',id=group,summary=True)['bounds']
 assert abs(group_bounds['x']+group_bounds['width']-500.0)<1e-7, group_bounds
 expected=[{'x':400.0,'y':100.0,'width':50.0,'height':100.0},{'x':460.0,'y':100.0,'width':40.0,'height':100.0}]
 for id,want in zip([a,b],expected):
  assert command('document.node',id=id,summary=True)['bounds']==want
 results.append({'check':'group_drag_to_artboard','group':node(group),'children_before':before,'children_after':moved})
 command('edit.undo')
 restored=[node(a),node(b)]
 assert restored==before, 'Undo did not restore children exactly'
 command('edit.redo')
 assert [node(a),node(b)]==moved, 'Redo did not restore moved geometry'
 results.append({'check':'group_undo_redo','pass':True})
 command('edit.undo')
 command('select.set',ids=[group])
 original_group=node(group)
 call('ui.pointer',{'events':[{'kind':'down','x':125,'y':125},{'kind':'drag','x':424,'y':125},{'kind':'up','x':424,'y':125}],'mods':{'shift':True,'alt':True}})
 copies=call('document.inspect')['selection']
 assert len(copies)==1 and copies[0]!=group, copies
 copied=command('document.node',id=copies[0],summary=True)['bounds']
 assert copied=={'x':400.0,'y':100.0,'width':100.0,'height':100.0}, copied
 assert node(group)==original_group, 'Alt-copy moved the original group'
 assert [node(a),node(b)]==before, 'Alt-copy changed original child geometry'
 results.append({'check':'Shift_Alt_group_copy','pass':True,'copy_bounds':copied,'original_unchanged':True})
 output={'executable_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'results':results,'view_flags':{k:settings[k] for k in ['smart_guides','snap_to_grid','snap_to_point','snap_to_pixel']}}
 (exe.parent/'RUNTIME_SMOKE.json').write_text(json.dumps(output,ensure_ascii=False,indent=2),encoding='utf-8')
 print(json.dumps({'sha256':output['executable_sha256'],'passed':['group_to_artboard_exact_500','rigid_children','undo','redo','Shift_Alt_copy_original_unchanged'],'view_flags':output['view_flags']},ensure_ascii=False))
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
