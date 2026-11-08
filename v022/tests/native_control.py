"""Isolated native-app control for synthetic Windows regression documents."""
import json,socket,subprocess,time
from pathlib import Path

class NativeControl:
    def __init__(self,exe):self.exe=Path(exe).resolve();self.connection=None;self.serial=0
    def __enter__(self):
        assert (self.exe.parent/'portable.txt').exists()
        startup=subprocess.STARTUPINFO();startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=0
        probe=socket.socket();probe.bind(('127.0.0.1',0));port=probe.getsockname()[1];probe.close()
        self.proc=subprocess.Popen([str(self.exe),'--new-instance','--control',str(port),'--automation-read-root',str(self.exe.parent),'--automation-write-root',str(self.exe.parent)],cwd=self.exe.parent,startupinfo=startup,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        deadline=time.monotonic()+90
        while time.monotonic()<deadline:
            if self.proc.poll() is not None:raise RuntimeError('Own test app exited')
            try:self.connection=socket.create_connection(('127.0.0.1',port),timeout=2);break
            except OSError:time.sleep(.5)
        if self.connection is None:
            self.proc.terminate();self.proc.wait(timeout=10);raise TimeoutError('Control startup')
        self.connection.settimeout(60);self.stream=self.connection.makefile('rwb');return self
    def call(self,method,params=None):
        self.serial+=1;self.stream.write((json.dumps({'id':self.serial,'method':method,'params':params or {}})+'\n').encode());self.stream.flush()
        r=json.loads(self.stream.readline())
        if not r.get('ok'):raise RuntimeError(r)
        return r.get('result')
    def cmd(self,name,**p):return self.call('engine.execute',{'command':name,'params':p})
    def node(self,id):return self.cmd('document.node',id=id)
    def bounds(self,id):return self.cmd('document.node',id=id,summary=True)['bounds']
    def __exit__(self,*exc):
        try:
            self.cmd('file.closeAll')
            for _ in range(128):self.call('ui.dialog.set',{'field':'discard','value':True});self.call('ui.dialog.confirm')
        except Exception:pass
        try:self.call('app.quit');self.proc.wait(timeout=10)
        except Exception:pass
        if self.connection is not None:self.connection.close()
        if self.proc.poll() is None:self.proc.terminate();self.proc.wait(timeout=10)
