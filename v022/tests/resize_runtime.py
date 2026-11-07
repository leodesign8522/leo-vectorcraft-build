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
 command('path.create',d=' '.join(f'M{10000+i*20} {10000+i*15}h10v10h-10Z' for i in range(5001)))
 source=command('shape.rectangle',x=400,y=300,width=300,height=200)['id']
 for p in [(200,0,200,900),(900,0,900,900),(100,150,1000,150),(100,700,1000,700)]:
  command('shape.line',x1=p[0],y1=p[1],x2=p[2],y2=p[3])
 command('view.setZoom',zoom=100,center=[550,400])
 call('ui.tool.select',{'tool':'selection'})
 original=node(source)
 def bounds():return command('document.node',id=source,summary=True)['bounds']
 cases=[('left',[400,400],[204,400],{'x':200,'y':300,'width':500,'height':200}),('right',[700,400],[896,400],{'x':400,'y':300,'width':500,'height':200}),('top',[550,300],[550,154],{'x':400,'y':150,'width':300,'height':350}),('bottom',[550,500],[550,696],{'x':400,'y':300,'width':300,'height':400})]
 records=[]
 for mode in ['none','grid','pixel']:
  flags=call('ui.inspect')['ui']['view']
  for key,cmd,wanted in [('snap_to_grid','view.snapToGrid',mode=='grid'),('snap_to_pixel','view.snapToPixel',mode=='pixel')]:
   if flags[key]!=wanted:command(cmd)
  for name,start,pointer,want in cases:
   command('select.set',ids=[source])
   def events(items):return call('ui.pointer',{'events':[{'kind':k,'x':p[0],'y':p[1]} for k,p in items]})
   events([('down',start),('drag',pointer)])
   got=bounds();passed=all(abs(got[k]-v)<1e-7 for k,v in want.items())
   records.append({'mode':mode,'handle':name,'pass':passed,'bounds':got})
   if '--observe' not in sys.argv:assert passed,records[-1]
   if '--capture' in sys.argv and name=='left' and mode=='none':call('ui.screenshot',{'path':str(exe.parent/'RESIZE_LEFT.png'),'data':False})
   events([('up',pointer)]);committed=node(source)
   command('edit.undo');assert node(source)==original
   command('edit.redo');assert node(source)==committed
   command('edit.undo')
 output={'pass':all(x['pass'] for x in records),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'checks':records,'background_anchors':20004,'undo_redo':True}
 (exe.parent/'RESIZE_RUNTIME.json').write_text(json.dumps(output,indent=2),encoding='utf-8')
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
