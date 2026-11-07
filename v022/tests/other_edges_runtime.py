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


 import math
 command('file.new',width=1400,height=1200)
 call('ui.set',{'smartGuides':True})
 flags=call('ui.inspect')['ui']['view']
 for flag,cmd in [('snap_to_grid','view.snapToGrid'),('snap_to_pixel','view.snapToPixel')]:
  if flags[flag]:command(cmd)
 data=' '.join(f'M{10000+i*20} {10000+i*15}h10v10h-10Z' for i in range(5001))
 command('path.create',d=data)
 points=[[300,300],[650,380],[580,650],[200,520]]
 polygon=command('path.create',d='M300 300L650 380L580 650L200 520Z')['id']
 command('view.setZoom',zoom=100,center=[450,450])
 original=node(polygon)
 call('ui.tool.select',{'tool':'directSelection'})
 checks=[]
 def events(items):return call('ui.pointer',{'events':[{'kind':kind,'x':p[0],'y':p[1]} for kind,p in items]})
 for ai in range(4):
  for neighbor in [(ai-1)%4,(ai+1)%4]:
   a,b=points[ai],points[neighbor];dx,dy=a[0]-b[0],a[1]-b[1];length=math.hypot(dx,dy);u=[dx/length,dy/length]
   for amount in [100,-80]:
    want=[a[0]+u[0]*amount,a[1]+u[1]*amount];pointer=[want[0]-u[1]*3,want[1]+u[0]*3]
    command('select.anchors',id=polygon,anchors=[[0,ai]],mode='set')
    events([('down',a),('drag',pointer)])
    after=node(polygon)['kind']['path']['subpaths'][0]['anchors']
    assert math.dist(after[ai]['p'],want)<1e-7,(ai,neighbor,amount,after)
    for other in range(4):
     if other!=ai: assert after[other]==original['kind']['path']['subpaths'][0]['anchors'][other]
    events([('up',pointer)])
    committed=node(polygon);command('edit.undo');assert node(polygon)==original
    command('edit.redo');assert node(polygon)==committed
    command('edit.undo');checks.append({'corner':ai,'edge_neighbor':neighbor,'direction':amount,'pass':True})
 source=command('shape.line',x1=600,y1=600,x2=500,y2=400)['id']
 target=command('shape.line',x1=900,y1=100,x2=900,y2=200)['id']
 original_source=node(source);original_target=node(target)
 # The next cases explicitly use 100% to check screen-pixel release distances.
 command('view.setZoom',zoom=100,center=[850,250])
 for y in [350,50]:
  command('select.anchors',id=source,anchors=[[0,1]],mode='set')
  events([('down',[500,400]),('drag',[904,y])])
  def endpoint():return node(source)['kind']['path']['subpaths'][0]['anchors'][1]['p']
  assert math.dist(endpoint(),[900,y])<1e-7,('other object extension',endpoint(),y)
  if y==350:
   events([('drag',[910,y])]);assert math.dist(endpoint(),[900,y])<1e-7
   events([('drag',[913,y])]);assert math.dist(endpoint(),[913,y])<1e-7
   events([('drag',[904,y])]);assert math.dist(endpoint(),[900,y])<1e-7
  if '--capture' in sys.argv:call('ui.screenshot',{'path':str(exe.parent/('YELLOW_OTHER_'+str(y)+'.png')),'data':False})
  events([('up',[904,y])]);committed=node(source)
  assert node(target)==original_target
  command('edit.undo');assert node(source)==original_source
  command('edit.redo');assert node(source)==committed
  command('edit.undo')
 command('view.setZoom',zoom=100,center=[850,400])
 command('select.anchors',id=polygon,anchors=[[0,3]],mode='set')
 direction=[350/math.hypot(350,80),80/math.hypot(350,80)]
 want=[1000,460];pointer=[want[0]-direction[1]*3,want[1]+direction[0]*3]
 events([('down',points[3]),('drag',pointer)])
 after=node(polygon)['kind']['path']['subpaths'][0]['anchors']
 assert math.dist(after[3]['p'],want)<1e-7,('non-adjacent original edge',after[3])
 if '--capture' in sys.argv:call('ui.screenshot',{'path':str(exe.parent/'YELLOW_NONADJACENT.png'),'data':False})
 events([('up',pointer)]);command('edit.undo');assert node(polygon)==original
 # A foreign supporting line near the cursor must not acquire while its real
 # segment is outside the viewport. Zooming out reveals it during the same drag.
 command('file.new',width=1400,height=1200)
 source=command('shape.line',x1=600,y1=600,x2=500,y2=400)['id']
 target=command('shape.line',x1=900,y1=-1800,x2=900,y2=-1700)['id']
 before=node(source)
 command('view.setZoom',zoom=100,center=[850,250])
 call('ui.tool.select',{'tool':'directSelection'})
 command('select.anchors',id=source,anchors=[[0,1]],mode='set')
 events([('down',[500,400]),('drag',[904,350])])
 assert math.dist(endpoint(),[904,350])<1e-7,('offscreen reference stole capture',endpoint())
 command('view.setZoom',zoom=10,center=[850,-500])
 events([('drag',[904,350])]);assert math.dist(endpoint(),[900,350])<1e-7,('revealed reference did not capture',endpoint())
 command('view.setZoom',zoom=100,center=[850,250])
 events([('drag',[910,350])]);assert math.dist(endpoint(),[900,350])<1e-7,('held reference lost on pan',endpoint())
 events([('drag',[913,350])]);assert math.dist(endpoint(),[913,350])<1e-7
 events([('drag',[904,350])]);assert math.dist(endpoint(),[904,350])<1e-7,('offscreen reference reacquired',endpoint())
 events([('up',[904,350])]);command('edit.undo');assert node(source)==before
 output={'pass':True,'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'corner_cases':checks,'other_object_both_rays':True,'hold_release':True,'target_unchanged':True,'non_adjacent_edge':True,'background_anchors':20004,'viewport_only':True,'view_updates_during_drag':True,'held_target_survives_pan':True}
 (exe.parent/'OTHER_EDGES_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8')
 print(json.dumps({'pass':True,'corner_cases':len(checks),'other_object_both_rays':True,'non_adjacent_edge':True,'hold_release':True,'undo_redo':True,'viewport_only':True,'view_updates_during_drag':True,'held_target_survives_pan':True}))

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
