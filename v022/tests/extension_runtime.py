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

 command('file.new',width=1000,height=1000)
 call('ui.set',{'smartGuides':True})
 flags=call('ui.inspect')['ui']['view']
 if flags['snap_to_grid']: command('view.snapToGrid')
 command('path.create',d='M10000 10000H10010V10010H10000Z '*5001)
 source=command('path.create',d='M300 300L600 330L600 500L460 420Z')['id']
 command('view.setZoom',zoom=100,center=[450,400])
 original=node(source)
 call('ui.tool.select',{'tool':'directSelection'})
 checks=[]
 for pointer,want in [([541.5,478.0],[540.0,480.0]),([381.5,358.0],[380.0,360.0]),([221.5,238.0],[220.0,240.0])]:
  command('select.anchors',id=source,anchors=[[0,3]],mode='set')
  call('ui.pointer',{'events':[{'kind':'down','x':460,'y':420}]})
  if not checks:
   call('ui.pointer',{'events':[{'kind':'drag','x':520,'y':440}]})
   assert node(source)['kind']['path']['subpaths'][0]['anchors'][3]['p']==[520.0,440.0]
  call('ui.pointer',{'events':[{'kind':'drag','x':pointer[0],'y':pointer[1]}]})
  preview=node(source)['kind']['path']['subpaths'][0]['anchors']
  actual=preview[3]['p']
  assert max(abs(actual[i]-want[i]) for i in [0,1])<1e-7, ('during drag',actual,want)
  assert preview[:3]==original['kind']['path']['subpaths'][0]['anchors'][:3]
  if not checks and '--capture' in sys.argv:
   call('ui.screenshot',{'path':str(exe.parent/'EXTENSION_HIT.png'),'data':False})
  call('ui.pointer',{'events':[{'kind':'up','x':pointer[0],'y':pointer[1]}]})
  committed=node(source)
  assert committed['kind']['path']['subpaths'][0]['anchors']==preview
  command('edit.undo'); assert node(source)==original
  command('edit.redo'); assert node(source)==committed
  command('edit.undo')
  checks.append({'endpoint':actual,'commit_undo_redo':True,'stationary_nodes_unchanged':True})
 output={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':checks}
 (exe.parent/'EXTENSION_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8')
 print(json.dumps(output))

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
