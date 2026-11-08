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
 command('file.new',width=1200,height=1200)
 call('ui.set',{'smartGuides':True})
 command('path.create',d=' '.join(f'M{10000+i*20} {10000+i*15}h10v10h-10Z' for i in range(5001)))
 source=command('path.create',d='M200 600L600 600L600 700L200 700Z')['id']
 target=command('shape.line',x1=600,y1=600,x2=200,y2=400)['id']
 # A closer angular match outside the viewport must not win acquisition.
 want=math.atan(.5)
 command('shape.line',x1=20000,y1=20000,x2=20000+400*math.cos(want+.01),y2=20000+400*math.sin(want+.01))
 command('view.setZoom',zoom=100,center=[400,550])
 original=node(source);target_before=node(target)
 points=[[200,600],[600,600],[600,700],[200,700]]
 origin=[600,600];radius=math.hypot(400,100)
 def rotated(p,angle,o=origin):
  x,y=p[0]-o[0],p[1]-o[1]
  return [o[0]+math.cos(angle)*x-math.sin(angle)*y,o[1]+math.sin(angle)*x+math.cos(angle)*y]
 def pointer(angle,o=origin):return rotated([o[0]-400,o[1]],angle,o)
 def points_of(id):return [a['p'] for a in node(id)['kind']['path']['subpaths'][0]['anchors']]
 def matches(id,angle,o=origin):return all(math.dist(a,rotated(b,angle,o))<1e-7 for a,b in zip(points_of(id),points))
 def events(items,mods=None):return call('ui.pointer',{'events':[{'kind':k,'x':p[0],'y':p[1]} for k,p in items],'mods':mods or {}})
 def begin(o=origin,start=[200,600],mods=None):
  command('select.set',ids=[source]);call('ui.tool.select',{'tool':'rotate'})
  events([('down',o),('up',o)])
  events([('down',start)],mods)
 records=[]
 for mode in ['none','grid','pixel']:
  flags=call('ui.inspect')['ui']['view']
  for key,cmd,wanted in [('snap_to_grid','view.snapToGrid',mode=='grid'),('snap_to_pixel','view.snapToPixel',mode=='pixel')]:
   if flags[key]!=wanted:command(cmd)
  for kind,mods in [('normal',{}),('alt',{'alt':True}),('shift',{'shift':True})]:
   begin(mods=mods);p=pointer(want+.01)
   events([('drag',p)],mods)
   selected=call('document.inspect')['selection'];moved=selected[0]
   expected=math.pi/4 if kind=='shift' else want
   passed=matches(moved,expected)
   records.append({'mode':mode,'kind':kind,'pass':passed})
   if '--observe' not in sys.argv:assert passed,records[-1]
   if kind=='alt':assert moved!=source and node(source)==original
   assert node(target)==target_before
   if '--capture' in sys.argv and mode=='none' and kind=='normal':call('ui.screenshot',{'path':str(exe.parent/'ROTATE_COLLINEAR.png'),'data':False})
   events([('up',p)],mods);committed=node(moved)
   command('edit.undo');assert node(source)==original
   command('edit.redo');assert node(moved)==committed
   command('edit.undo')
 extras={}
 if '--observe' not in sys.argv:
  flags=call('ui.inspect')['ui']['view']
  for key,cmd in [('snap_to_grid','view.snapToGrid'),('snap_to_pixel','view.snapToPixel')]:
   if flags[key]:command(cmd)
  begin()
  for px,hit in [(7.9,True),(12.7,True),(12.9,False),(8.1,False),(7.9,True)]:
   raw=want+2*math.asin(px/(2*radius));events([('drag',pointer(raw))])
   assert matches(source,want if hit else raw),(px,points_of(source))
  events([('up',pointer(raw))]);command('edit.undo');assert node(source)==original
  # Direct Selection may leave every anchor selected before switching to R.
  command('select.anchors',id=source,anchors=[[0,i] for i in range(4)],mode='set')
  call('ui.tool.select',{'tool':'rotate'});events([('down',origin),('up',origin),('down',[200,600]),('drag',pointer(want+.01))])
  assert matches(source,want)
  events([('up',pointer(want+.01))]);command('edit.undo');assert node(source)==original
  # A different fixed pivot can align directions without translating the shape.
  pivot=[600,700];begin(pivot,[200,700]);p=pointer(want+.01,pivot)
  events([('drag',p)]);assert matches(source,want,pivot)
  assert math.dist(points_of(source)[2],pivot)<1e-7
  if '--capture' in sys.argv:call('ui.screenshot',{'path':str(exe.parent/'ROTATE_PARALLEL.png'),'data':False})
  events([('up',p)]);command('edit.undo');assert node(source)==original
  # A group receives one rigid rotation about the same user-selected pivot.
  child=command('shape.line',x1=700,y1=800,x2=750,y2=850)['id']
  command('select.set',ids=[source,child]);group=command('object.group')['id']
  group_before=node(group);child_before=points_of(child)
  call('ui.tool.select',{'tool':'rotate'});events([('down',origin),('up',origin),('down',[200,600]),('drag',pointer(want+.01))])
  assert matches(source,want)
  assert all(math.dist(a,rotated(b,want))<1e-7 for a,b in zip(points_of(child),child_before))
  events([('up',pointer(want+.01))]);group_after=node(group)
  command('edit.undo');assert node(group)==group_before
  command('edit.redo');assert node(group)==group_after
  command('edit.undo')
  extras={'hysteresis':True,'custom_pivot_fixed':True,'parallel_direction':True,'offscreen_reference_excluded':True,'group_rigid_rotation':True,'full_anchor_rotation':True}
 output={'pass':all(x['pass'] for x in records),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':records,'background_anchors':20004,'target_unchanged':True,'alt_original_unchanged':True,'undo_redo':True,**extras}
 (exe.parent/'ROTATION_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8');print(json.dumps(output))

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
